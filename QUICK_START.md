# RustyEngine - Quick Start & Implementation Checklist

## 📋 Quick Reference

### Architecture Score: 7.8/10 ⭐⭐
- ✅ Excellent modularity & trait design
- ✅ Events system complete
- ⚠️ Error handling needs work
- ⚠️ WgpuRenderer incomplete
- ⚠️ No sprite batching

### Immediate Actions (Next 2 weeks)

1. **RenderError** → Add diagnostic details
2. **Input::frame_reset()** → Fix just_pressed decay
3. **Asset unload** → Add memory management
4. **Sprite batching** → Critical for performance
5. **Tests** → Add integration tests

---

## 🔧 Phase 1 Implementation Checklist (1-2 Weeks)

### [ ] 1.1 - Improve RenderError
- **File**: `src/render/mod.rs`
- **Change**: `pub struct RenderError;` → `pub enum RenderError { ... }`
- **Add**: 8+ error variants with thiserror
- **Add**: Logging with `log::error!()`
- **Time**: 1 hour
- **Status**: ⬜ NOT STARTED

```rust
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to initialize renderer: {0}")]
    InitFailed(String),
    // ... 7 more variants
}
```

**Verify**:
```bash
cargo check
cargo clippy
cargo test
```

---

### [ ] 1.2 - Implement Input::frame_reset()
- **File**: `src/core/events/input.rs`
- **Changes**:
  - Add `pub fn frame_reset(&mut self)` method
  - Add `pub fn is_key_held(&self, key: Key) -> bool`
  - Add `pub fn is_key_pressed(&self, key: Key) -> bool`
  - Add `pub fn any_key_pressed(&self) -> bool`
- **File**: `src/core/events/event_handler.rs`
  - Call `self.input.frame_reset()` at start of `on_update()`
- **Time**: 45 minutes
- **Status**: ⬜ NOT STARTED

```rust
// In event_handler.rs on_update():
fn on_update(&mut self, state: &EngineState) {
    self.input.frame_reset();  // ← ADD THIS LINE FIRST
    self.on_update.invoke(state);
}
```

**Verify**: Write test checking just_pressed cleared after frame

---

### [ ] 1.3 - Add Frame Limiting
- **File**: `src/backend/winit_backend.rs`
- **Changes**:
  - Add `target_frame_time: Option<Duration>` field
  - Add `pub fn set_target_fps(&mut self, fps: u32)` method
  - Implement sleep logic in render loop
  - Hook WindowConfig::target_fps
- **Time**: 1 hour
- **Status**: ⬜ NOT STARTED

```rust
// In WinitBackend:
pub fn set_target_fps(&mut self, fps: u32) {
    if fps > 0 {
        self.target_frame_time = Some(Duration::from_secs_f64(1.0 / fps as f64));
    }
}

fn frame_limiter_sleep(&self) {
    if let Some(target_time) = self.target_frame_time {
        let elapsed = self.last_frame.elapsed();
        if elapsed < target_time {
            std::thread::sleep(target_time - elapsed);
        }
    }
}
```

**Verify**:
```bash
# Run and check FPS is capped
cargo run --release
```

---

### [ ] 1.4 - Asset Lifecycle Management
- **File**: `src/core/assets/manager.rs`
- **Changes**:
  - Add `max_memory_bytes` field
  - Add `current_memory_bytes` tracking
  - Modify `load_image()` to check memory limit
  - Add `pub fn unload_image(&mut self, id: ImageId) -> bool`
  - Add `pub fn memory_usage(&self) -> usize`
  - Add `pub fn clear_all(&mut self)`
- **File**: `src/core/assets/error.rs`
  - Add `MemoryExceeded` variant
- **Time**: 1.5 hours
- **Status**: ⬜ NOT STARTED

```rust
pub struct AssetManager {
    images: HashMap<ImageId, ImageAsset>,
    max_memory_bytes: usize,
    current_memory_bytes: usize,
}

impl AssetManager {
    pub fn with_limit(max_bytes: usize) -> Self {
        // ...
    }
    
    pub fn unload_image(&mut self, id: ImageId) -> bool {
        // Remove & update tracking
    }
}
```

**Verify**:
```bash
cargo test core::assets
```

---

### [ ] 1.5 - Enhanced Error Types
- **File**: `src/audio/error.rs`
- **Changes**: Improve `AudioError` enum (already good, just expand)
  - Add more variants
  - Better messages
- **File**: `src/backend/window_backend.rs`
  - Already uses thiserror, good
- **Time**: 30 minutes
- **Status**: ⬜ NOT STARTED

---

### [ ] 1.6 - Fix Compilation Warnings
- **Run**:
  ```bash
  cargo clippy -- -D warnings
  cargo build --all-targets
  ```
- **Fix**: All warnings
- **Time**: 30 minutes - 1 hour
- **Status**: ⬜ NOT STARTED

---

### [ ] 1.7 - Add Basic Unit Tests
- **File**: Create or update `tests/`
- **Tests needed**:
  - `test_asset_memory_limit`
  - `test_input_frame_reset`
  - `test_window_config_validation`
  - `test_audio_error_handling`
- **Time**: 1.5 hours
- **Status**: ⬜ NOT STARTED

```bash
cargo test --lib
cargo test --test '*'
```

---

### 📊 Phase 1 Progress Tracker

| Task | Complexity | Time | Status |
|------|-----------|------|--------|
| 1.1 RenderError | 🟢 Easy | 1h | ⬜ |
| 1.2 Input reset | 🟢 Easy | 45m | ⬜ |
| 1.3 Frame limit | 🟡 Medium | 1h | ⬜ |
| 1.4 Asset mgmt | 🟡 Medium | 1.5h | ⬜ |
| 1.5 Error types | 🟢 Easy | 30m | ⬜ |
| 1.6 Clippy fixes | 🟢 Easy | 1h | ⬜ |
| 1.7 Unit tests | 🟡 Medium | 1.5h | ⬜ |
| **TOTAL** | | **7.5h** | |

**Target**: Complete Phase 1 in 1 week (2h/day)

---

## 🚀 Phase 2 Implementation Checklist (2-4 Weeks)

### [ ] 2.1 - Sprite Batching
- **File**: `src/render/wgpu_renderer.rs`
- **Complexity**: 🔴 HIGH
- **Time**: 3-4 hours
- **Key Changes**:
  - Remove `sprite_draws: Vec<SpriteDraw>`
  - Add `sprite_batches: Vec<SpriteBatch>`
  - Implement batching by texture_id + z_order
  - Single drawcall per texture (instead of per-sprite)
- **Expected Perf**: 1000+ drawcalls → 5-10 drawcalls
- **Status**: ⬜ NOT STARTED

```rust
struct SpriteBatch {
    texture_id: ImageId,
    vertices: Vec<SpriteVertexGPU>,
    index_offset: u32,
    index_count: u32,
}

// Expected: 10k sprites at 60 FPS ✅
```

---

### [ ] 2.2 - Integration Tests
- **Create**: `tests/integration_tests.rs`
- **Tests**:
  - Window creation
  - Asset loading
  - Renderer initialization
  - Full game loop (stub)
- **Time**: 2 hours
- **Status**: ⬜ NOT STARTED

```bash
cargo test --test integration_tests --release
```

---

### [ ] 2.3 - Examples
- **Create**: `examples/` folder
  - `sprite_demo.rs` - Load & render sprites
  - `audio_demo.rs` - Play sounds
  - `input_demo.rs` - Handle events
  - `shapes_demo.rs` - Draw primitives
- **Time**: 2-3 hours
- **Status**: ⬜ NOT STARTED

```bash
cargo run --example sprite_demo --release
```

---

### [ ] 2.4 - Documentation
- **Update**: README.md
  - Quick start (5 minutes)
  - Architecture overview
  - API examples
  - Troubleshooting
- **Add**: rustdoc comments to all public APIs
- **Time**: 2 hours
- **Status**: ⬜ NOT STARTED

```bash
cargo doc --no-deps --open
```

---

### [ ] 2.5 - Performance Benchmarking
- **Create**: `benches/` folder
- **Benchmarks**:
  - Sprite rendering (1000, 5000, 10000 sprites)
  - Asset loading times
  - Event dispatching overhead
- **Tool**: `cargo bench --release`
- **Target**: 10k sprites @ 60 FPS
- **Time**: 1.5 hours
- **Status**: ⬜ NOT STARTED

---

## 🎯 Phase 3 Enhancements (4-8 Weeks)

### [ ] 3.1 - Scene Graph
- **Files**: `src/core/scene.rs`, `src/core/transform.rs`
- **Features**:
  - SceneNode hierarchy
  - Transform composition
  - Component system
- **Time**: 4-5 hours
- **Status**: ⬜ NOT STARTED

---

### [ ] 3.2 - Animation System
- **File**: `src/core/animation.rs`
- **Features**:
  - Keyframe animation
  - Sprite sheet support
  - Easing curves
- **Time**: 3-4 hours
- **Status**: ⬜ NOT STARTED

---

### [ ] 3.3 - Basic Physics
- **File**: `src/physics/` (new module)
- **Features**:
  - 2D rigid bodies
  - Gravity
  - Collision callbacks
- **Time**: 5-6 hours
- **Status**: ⬜ NOT STARTED

---

## 📝 Commit Message Template

```
feat: [module] brief description

Type: feat|fix|refactor|docs|test|perf

Module: render|audio|core|backend|assets|events

Details:
- What changed
- Why it changed
- Impact on performance/API

Closes #123
```

Example:
```
feat: render - implement sprite batching

Type: perf
Module: render

- Batch sprites by texture_id + z_order
- Reduce drawcalls: 1000+ → 5-10
- Expected: 10k sprites @ 60 FPS on mid-range GPU

Perf: +5-10x FPS improvement with large sprite counts
Breaking: None
```

---

## 🧪 Testing Commands

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# All tests with output
cargo test -- --nocapture

# Specific test
cargo test test_asset_memory_limit

# Test with logging
RUST_LOG=debug cargo test -- --nocapture

# Bench
cargo bench --release
```

---

## 🐛 Debugging Helpers

```bash
# Find warnings
cargo clippy

# Strict linting
cargo clippy -- -D warnings

# Security audit
cargo audit

# Documentation check
cargo doc --no-deps

# Code coverage (requires tarpaulin)
cargo tarpaulin --out Html

# Flamegraph profiling
cargo flamegraph --release --example sprite_demo
```

---

## 📦 Dependency Audit

**Current deps** (good):
```toml
wgpu = "27.0.1"        ✅ Modern GPU API
winit = "0.30.12"      ✅ Window/events
thiserror = "2.0.17"   ✅ Error handling
log = "0.4.29"         ✅ Logging
pollster = "0.4.0"     ✅ Async runtime
rodio = "0.17.1"       ✅ Audio
image = "0.25"         ✅ Image loading
```

**Potential additions** (Phase 2+):
```toml
serde = "1.0"          📦 Serialization (assets)
uuid = "1.0"           📦 Better IDs
parking_lot = "0.12"   ⚡ Faster locks
rayon = "1.7"          ⚡ Parallelization
```

---

## 🎓 Learning Resources

### For This Codebase:
1. Read `ARCHITECTURE_ANALYSIS.md` (overview)
2. Read `IMPROVEMENT_PLAN.md` (detailed actions)
3. Study `src/core/engine.rs` (entry point)
4. Study `src/render/renderer.rs` (trait design)
5. Study `src/core/events/event_handler.rs` (event system)

### Rust Resources:
- https://doc.rust-lang.org/book/ (Rust basics)
- https://docs.rs/wgpu/ (GPU API)
- https://docs.rs/winit/ (Window/events)
- https://doc.rust-lang.org/reference/ (Advanced)

### Game Dev Resources:
- Game Architecture Patterns (Robert Nystrom)
- Real-Time Rendering (Akenine-Moeller et al.)
- Entity Component System (bevy_ecs docs)

---

## 🎯 Success Criteria

### Phase 1 (2 weeks):
- [ ] RenderError detailed
- [ ] Input frame_reset working
- [ ] Frame limiting functional
- [ ] Asset unload available
- [ ] `cargo test` all pass
- [ ] `cargo clippy` warnings = 0

### Phase 2 (4 weeks):
- [ ] 10k sprites @ 60 FPS ✅
- [ ] Integration tests passing
- [ ] Examples runnable
- [ ] README complete
- [ ] Doc coverage > 80%

### Phase 3 (8 weeks):
- [ ] Scene graph operational
- [ ] Animation system working
- [ ] Physics basic integrated
- [ ] All phase 1-2 stable
- [ ] Ready for beta release

---

## 🚦 Progress Dashboard

**Last Updated**: 2025-12-23

```
Phase 1 (Critical):   ▯▯▯▯▯▯▯▯▯▯ 0% (Starting)
Phase 2 (Important):  ▯▯▯▯▯▯▯▯▯▯ 0%
Phase 3 (Enhancements): ▯▯▯▯▯▯▯▯▯▯ 0%

Overall: ▯▯▯▯▯▯▯▯▯▯ 0%
Status: 🟡 Ready for Phase 1 Start
```

**Update this after each major milestone**

---

## 📞 Questions & Troubleshooting

### "I'm new to Rust, will I understand RustyEngine?"
- **Answer**: Yes, but spend 1-2 weeks on Rust basics first
- **Recommend**: Rust Book chapters 1-10
- **Then**: Start with Asset/Error improvements (Phase 1.1-1.5)

### "How do I test my changes?"
- **Answer**: See Testing Commands section above
- **Quick**: `cargo test && cargo clippy`
- **Full**: `cargo test --all && cargo bench --release`

### "Where should I start?"
- **Answer**: Phase 1.1 (RenderError) - good intro
- **Then**: 1.2, 1.3, 1.4 in order
- **Time**: ~2 hours each, doable in 1 week

### "Will these changes break existing code?"
- **Answer**: Phase 1 mostly additive, minimal breaks
- **Exception**: RenderError enum → users must handle variants
- **Mitigation**: Update examples alongside

---

## 📅 Recommended Schedule

### Week 1 (Phase 1.1-1.3):
- Mon-Tue: RenderError + clippy fixes
- Wed: Input::frame_reset + tests
- Thu: Frame limiting
- Fri: Polish + verification

### Week 2 (Phase 1.4-1.7):
- Mon: Asset lifecycle
- Tue: Audio error enhancements
- Wed-Thu: Unit tests
- Fri: Full test suite + benchmarks

### Week 3-4 (Phase 2.1-2.2):
- Sprite batching (most complex)
- Integration tests
- Examples

### Week 5-8 (Phase 2.3 + Phase 3):
- Documentation
- Scene graph (if time)
- Animation (if time)
- Polish for 1.0 beta

---

**Next Action**: Start Phase 1.1 (RenderError implementation)

**Time Investment**: 7-8 weeks for full completion (part-time: 2-3 hours/day)

**Expected Outcome**: Production-ready v1.0 beta with excellent architecture ✅

---

Good luck! 🚀
