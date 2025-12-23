# RustyEngine - Plan Amélioration Détaillé (Quick Actions)

## Phase 1: CRITIQUE (1-2 semaines)

### 1.1 ✅ Améliorer RenderError (Fichier: render/mod.rs)

**Avant** (Problème):
```rust
pub struct RenderError;
pub type RenderResult<T> = Result<T, RenderError>;
```

**Après** (Solution):
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to initialize renderer: {0}")]
    InitFailed(String),
    
    #[error("Shader compilation failed:\n{0}")]
    ShaderCompilation(String),
    
    #[error("GPU memory exhausted")]
    OutOfMemory,
    
    #[error("Device lost (GPU reset?)")]
    DeviceLost,
    
    #[error("Invalid texture format: {0}")]
    InvalidTexture(String),
    
    #[error("Rendering operation failed: {0}")]
    RenderFailed(String),
    
    #[error("Window surface error: {0}")]
    SurfaceError(String),
    
    #[error("Pipeline setup failed: {0}")]
    PipelineSetup(String),
}

pub type RenderResult<T> = Result<T, RenderError>;
```

**Changements dans wgpu_renderer.rs**:
- Replace `.map_err(|_| RenderError)` avec `.map_err(|e| RenderError::InitFailed(e.to_string()))`
- Add diagnostic logging avec `log::error!()` avant returning errors

---

### 1.2 ✅ Compléter Input State Reset (Fichier: core/events/input.rs)

Ajouter gestion "just_pressed/just_released decay":

```rust
pub struct Input {
    pub pressed_keys: HashSet<Key>,
    pub just_pressed: HashSet<Key>,
    pub just_released: HashSet<Key>,
    pub mouse_position: Position,
    pub mouse_delta: Vec2,
    pub just_mouse_pressed: bool,
    pub just_mouse_released: bool,
}

impl Input {
    /// Called at START of each frame - clears one-frame states
    pub fn frame_reset(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.just_mouse_pressed = false;
        self.just_mouse_released = false;
        self.mouse_delta = Vec2::ZERO;
    }
    
    /// Check if key is held DOWN (including this frame)
    pub fn is_key_held(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }
    
    /// Check if key was PRESSED this frame only
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.just_pressed.contains(&key)
    }
    
    /// Check if key was RELEASED this frame only
    pub fn is_key_released(&self, key: Key) -> bool {
        self.just_released.contains(&key)
    }
    
    /// Check if ANY key pressed this frame
    pub fn any_key_pressed(&self) -> bool {
        !self.just_pressed.is_empty()
    }
}
```

**Dans event_handler.rs - modifier on_update()**:
```rust
fn on_update(&mut self, state: &EngineState) {
    // ➕ AJOUTER CETTE LIGNE EN PREMIER:
    self.input.frame_reset();
    
    // Ensuite appeler les callbacks utilisateur
    self.on_update.invoke(state);
}
```

---

### 1.3 ✅ Implémenter Frame Limiting (Fichier: backend/winit_backend.rs)

Ajouter dans WinitBackend:

```rust
pub struct WinitBackend {
    // ... existant
    target_frame_time: Option<Duration>,
    last_frame: Instant,
}

impl WinitBackend {
    pub fn set_target_fps(&mut self, fps: u32) {
        if fps > 0 {
            self.target_frame_time = Some(Duration::from_secs_f64(1.0 / fps as f64));
        } else {
            self.target_frame_time = None;
        }
    }
    
    fn frame_limiter_sleep(&self) {
        if let Some(target_time) = self.target_frame_time {
            let elapsed = self.last_frame.elapsed();
            if elapsed < target_time {
                let sleep_time = target_time - elapsed;
                std::thread::sleep(sleep_time);
            }
        }
    }
}

// Dans run() loop:
// Appeler self.frame_limiter_sleep() après chaque redraw
```

---

### 1.4 ✅ Asset Lifecycle Management (Fichier: core/assets/manager.rs)

**Ajouter unload et memory tracking**:

```rust
pub struct AssetManager {
    images: HashMap<ImageId, ImageAsset>,
    max_memory_bytes: usize,
    current_memory_bytes: usize,
}

impl AssetManager {
    /// Create with max memory limit (e.g., 512MB)
    pub fn with_limit(max_bytes: usize) -> Self {
        Self {
            images: HashMap::new(),
            max_memory_bytes: max_bytes,
            current_memory_bytes: 0,
        }
    }
    
    /// Load image, with memory limit check
    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> AssetResult<ImageId> {
        let path_buf = path.as_ref().to_path_buf();
        let dyn_img = image::open(&path_buf).map_err(|source| AssetError::Image {
            source,
            path: path_buf.clone(),
        })?;
        let rgba = dyn_img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();
        
        let image_size = data.len(); // bytes
        if self.current_memory_bytes + image_size > self.max_memory_bytes {
            return Err(AssetError::MemoryExceeded);
        }
        
        let image = ImageAsset { width, height, data };
        let id = ImageId::new();
        
        self.images.insert(id, image);
        self.current_memory_bytes += image_size;
        
        Ok(id)
    }
    
    /// Unload specific image
    pub fn unload_image(&mut self, id: ImageId) -> bool {
        if let Some(image) = self.images.remove(&id) {
            self.current_memory_bytes -= image.data.len();
            true
        } else {
            false
        }
    }
    
    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.current_memory_bytes
    }
    
    /// Get memory limit
    pub fn memory_limit(&self) -> usize {
        self.max_memory_bytes
    }
}
```

**Aussi ajouter dans AssetError**:
```rust
#[derive(Debug, Error)]
pub enum AssetError {
    // ... existant
    #[error("Asset memory limit exceeded: {0} / {1} bytes")]
    MemoryExceeded(usize, usize),
}
```

---

### 1.5 ✅ Améliorer Audio Error Handling (Fichier: audio/error.rs)

```rust
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Failed to load audio: {path} - {source}")]
    LoadFailed {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Audio device unavailable: {0}")]
    NoDevice(String),
    
    #[error("Sound not found: {0:?}")]
    SoundNotFound(SoundId),
    
    #[error("Invalid volume level: {0} (must be 0.0-1.0)")]
    InvalidVolume(f32),
    
    #[error("Audio system not initialized")]
    NotInitialized,
    
    #[error("Backend error: {0}")]
    Backend(String),
}

pub type AudioResult<T> = Result<T, AudioError>;
```

---

## Phase 2: IMPORTANT (2-4 semaines)

### 2.1 ✅ Sprite Batching Optimization (Fichier: render/wgpu_renderer.rs)

**Problème actuel**: Chaque sprite = 1 drawcall (très inefficace)

```rust
/// Batch sprites by texture for efficient rendering
struct SpriteBatch {
    texture_id: ImageId,
    vertices: Vec<SpriteVertexGPU>,
    z_order: u32,
}

impl WgpuRenderer {
    /// Accumulate sprite draws for batching
    fn batch_sprite(&mut self, sprite: &Sprite, viewport_size: (u32, u32)) {
        // ... convert sprite to vertices
        // ... group by texture_id
        // ... sort by z_order
    }
    
    /// Flush all batches to GPU
    fn flush_sprite_batches(&mut self) -> RenderResult<()> {
        // Create single vertex buffer per texture
        // Single drawcall per texture
        // Clear batch
        Ok(())
    }
}

// Dans draw_sprites():
fn draw_sprites(&mut self, sprites: &[Sprite], viewport_size: (u32, u32)) {
    self.sprite_draws.clear();
    
    // Batch by texture
    for sprite in sprites {
        self.batch_sprite(sprite, viewport_size);
    }
    
    // Flush all batches
    if let Err(e) = self.flush_sprite_batches() {
        log::error!("Sprite batch flush failed: {}", e);
    }
}
```

**Expected Performance**: 1000+ drawcalls → 5-10 drawcalls

---

### 2.2 ✅ Testing Framework (Créer: tests/integration_tests.rs)

```rust
#[cfg(test)]
mod tests {
    use rusty_engine::*;
    
    #[test]
    fn test_window_creation() {
        // Setup
        let backend = WinitBackend::try_new().expect("Backend init");
        let renderer = WgpuRenderer::new();
        let mut engine = Engine::new(Box::new(backend), Box::new(renderer))
            .expect("Engine init");
        
        let config = WindowConfig::builder()
            .width(800)
            .height(600)
            .build();
        
        // Execute
        let result = engine.create_window(config);
        
        // Assert
        assert!(result.is_ok(), "Window creation failed");
    }
    
    #[test]
    fn test_asset_loading() {
        let mut assets = AssetManager::with_limit(100 * 1024 * 1024); // 100MB
        
        // Should work
        let result = assets.load_image("tests/fixtures/test.png");
        assert!(result.is_ok());
        
        // Should have memory tracked
        assert!(assets.memory_usage() > 0);
    }
    
    #[test]
    fn test_input_frame_reset() {
        let mut input = Input::new();
        input.just_pressed.insert(Key::A);
        
        input.frame_reset();
        
        assert!(input.just_pressed.is_empty());
    }
}
```

---

### 2.3 ✅ Documentation & Examples (Créer: examples/sprite_demo.rs)

```rust
use rusty_engine::prelude::*;

fn main() {
    env_logger::init();
    
    // Create backend and renderer
    let backend = Box::new(WinitBackend::try_new().expect("Failed to init winit"));
    let renderer = Box::new(WgpuRenderer::new());
    
    // Create engine
    let mut engine = Engine::new(backend, renderer).expect("Failed to init engine");
    
    // Create window
    let config = WindowConfig::builder()
        .width(1280)
        .height(720)
        .title("Sprite Demo")
        .build();
    engine.create_window(config).expect("Failed to create window");
    
    // Load asset
    let sprite_id = engine
        .assets
        .load_image("assets/player.png")
        .expect("Failed to load sprite");
    
    // Setup game loop
    engine.events.on_update.add(|state| {
        println!("FPS: {:.1}", state.fps);
    });
    
    engine.events.on_redraw.add(|| {
        println!("Rendering frame");
    });
    
    // Run
    if let Err(e) = engine.run() {
        eprintln!("Engine error: {}", e);
    }
}
```

---

## Phase 3: ENHANCEMENTS (4-8 semaines)

### 3.1 🔧 Scene Graph (Nouvelle architecture)

```rust
// core/scene.rs
pub struct Transform {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32, // radians
}

pub struct SceneNode {
    pub id: NodeId,
    pub transform: Transform,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
    pub components: Vec<Box<dyn Component>>,
}

pub trait Component: Send + Sync {
    fn update(&mut self, delta: Duration);
    fn render(&self, ctx: &mut RenderContext);
}

pub struct Scene {
    nodes: HashMap<NodeId, SceneNode>,
    root: NodeId,
}

impl Scene {
    pub fn add_sprite(&mut self, parent: NodeId, sprite: Sprite) -> NodeId {
        // ...
    }
}
```

---

### 3.2 🎬 Animation System

```rust
// core/animation.rs
pub struct Animation {
    pub frames: Vec<ImageId>,
    pub frame_duration: Duration,
    pub looping: bool,
}

pub struct Animator {
    animations: HashMap<String, Animation>,
    current: Option<String>,
    elapsed: Duration,
    frame_index: usize,
}

impl Animator {
    pub fn play(&mut self, name: &str) {
        self.current = Some(name.to_string());
        self.elapsed = Duration::ZERO;
        self.frame_index = 0;
    }
    
    pub fn update(&mut self, delta: Duration) {
        self.elapsed += delta;
        // ... advance frame_index based on frame_duration
    }
}
```

---

## Checklist d'Implémentation

### Phase 1 Checklist:
- [ ] RenderError enum complet avec thiserror
- [ ] Input::frame_reset() implémenté
- [ ] Frame limiting avec target_fps
- [ ] AssetManager avec lifecycle (unload, memory tracking)
- [ ] AudioError amélioré
- [ ] All compile && cargo test pass

### Phase 2 Checklist:
- [ ] Sprite batching implémenté
- [ ] Integration tests de base
- [ ] Examples folder avec 3+ exemples
- [ ] README.md documenté
- [ ] Perf benchmark (10k sprites à 60 FPS)

### Phase 3 Checklist:
- [ ] Scene graph optionnel
- [ ] Animation système
- [ ] Physics basique (si nécessaire)
- [ ] Python bindings exploratoires (pyo3)
- [ ] 1.0 release candidate

---

## Commandes de Validation

```bash
# Compiler strictement
cargo build --all-targets

# Tests unitaires
cargo test --lib

# Tests intégration
cargo test --test '*' --release

# Linting
cargo clippy -- -D warnings

# Formatting
cargo fmt --check

# Documentation
cargo doc --no-deps --open

# Profiling
cargo flamegraph --example sprite_demo --release

# Audit sécurité
cargo audit
```

---

## Timeline Réaliste

- **Week 1-2**: Phase 1 CRITIQUE (RenderError, Input, Frame limit, Assets, Audio)
- **Week 3-4**: Phase 2 IMPORTANT (Batching, Tests, Examples)
- **Week 5-8**: Phase 3 ENHANCEMENTS (Scene graph, Animation)
- **Week 9-12**: Polish, optimization, documentation, 1.0 beta
- **Total**: ~3 mois pour production-ready

---

**Next Steps**:
1. Start Phase 1 implementations (begin with RenderError)
2. Commit each change with clear messages
3. Run tests after each major change
4. Gather feedback from early users
5. Adjust plan based on bottlenecks discovered
