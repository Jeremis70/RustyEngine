# RustyEngine - Implementation TODO (Living Document)

**Last Updated**: 2025-12-23  
**Target Release**: v1.0 Beta (June 2026)  
**Current Phase**: 1 (Critical Fixes)

---

## 🎯 Phase 1: CRITICAL (Weeks 1-2) 

### Render System
- [ ] **1.1** RenderError enum implementation
  - [ ] Add 8+ error variants (InitFailed, DeviceLost, etc)
  - [ ] Update wgpu_renderer.rs error handling
  - [ ] Add log::error!() for diagnostics
  - [ ] Write test for error messages
  - **Owner**: TBD
  - **Due**: Week 1
  - **Effort**: 1-2 hours

- [ ] **1.2** WgpuRenderer completion
  - [ ] Complete init() method
  - [ ] Add device.poll() synchronization
  - [ ] Verify all render pipeline setup
  - [ ] Test on real GPU (Windows/Linux/macOS)
  - **Owner**: TBD
  - **Due**: Week 1-2
  - **Effort**: 3-4 hours

### Input System
- [ ] **1.3** Input::frame_reset() implementation
  - [ ] Add frame_reset() method
  - [ ] Add is_key_held() helper
  - [ ] Add is_key_pressed() helper
  - [ ] Call in event_handler.rs on_update()
  - [ ] Write tests for input state decay
  - **Owner**: TBD
  - **Due**: Week 1
  - **Effort**: 45 minutes

### Window Backend
- [ ] **1.4** Frame limiting support
  - [ ] Add target_frame_time to WinitBackend
  - [ ] Implement frame_limiter_sleep()
  - [ ] Hook WindowConfig::target_fps
  - [ ] Test FPS capping
  - **Owner**: TBD
  - **Due**: Week 1
  - **Effort**: 1 hour

### Asset Management
- [ ] **1.5** Asset lifecycle management
  - [ ] Add max_memory_bytes tracking
  - [ ] Implement unload_image()
  - [ ] Add memory_usage() getter
  - [ ] Handle MemoryExceeded error
  - [ ] Write memory limit tests
  - **Owner**: TBD
  - **Due**: Week 1-2
  - **Effort**: 1.5 hours

### Error Handling
- [ ] **1.6** Improve audio error types
  - [ ] Add AudioError variants
  - [ ] Better error messages
  - [ ] Add invalid volume validation
  - **Owner**: TBD
  - **Due**: Week 1
  - **Effort**: 30 minutes

### Code Quality
- [ ] **1.7** Fix all clippy warnings
  - [ ] Run: `cargo clippy -- -D warnings`
  - [ ] Fix all warnings
  - [ ] Verify: Zero warnings
  - **Owner**: TBD
  - **Due**: Week 2
  - **Effort**: 30-60 minutes

- [ ] **1.8** Basic unit tests
  - [ ] Test asset memory limit
  - [ ] Test input frame reset
  - [ ] Test window config validation
  - [ ] Test audio errors
  - [ ] Target: >70% coverage
  - **Owner**: TBD
  - **Due**: Week 2
  - **Effort**: 1.5 hours

### Verification
- [ ] All Phase 1 code compiles: `cargo build --all-targets`
- [ ] All tests pass: `cargo test --lib`
- [ ] Zero clippy warnings: `cargo clippy -- -D warnings`
- [ ] Documentation builds: `cargo doc --no-deps`

---

## 🔥 Phase 2: PERFORMANCE (Weeks 3-4)

### Rendering Optimization
- [ ] **2.1** Sprite batching implementation
  - [ ] Remove per-sprite drawcalls
  - [ ] Implement SpriteBatch struct
  - [ ] Batch by texture_id + z_order
  - [ ] Single drawcall per texture
  - [ ] Benchmark: 10k sprites @ 60 FPS
  - **Owner**: TBD
  - **Due**: Week 3
  - **Effort**: 3-4 hours

### Testing
- [ ] **2.2** Integration tests
  - [ ] Test window creation
  - [ ] Test asset loading
  - [ ] Test renderer init
  - [ ] Test event loop (stub)
  - [ ] Create tests/integration_tests.rs
  - **Owner**: TBD
  - **Due**: Week 3
  - **Effort**: 2 hours

### Documentation & Examples
- [ ] **2.3** Create examples/
  - [ ] sprite_demo.rs (load & render)
  - [ ] audio_demo.rs (play sounds)
  - [ ] input_demo.rs (handle events)
  - [ ] shapes_demo.rs (draw primitives)
  - [ ] All examples must compile & run
  - **Owner**: TBD
  - **Due**: Week 3-4
  - **Effort**: 2-3 hours

- [ ] **2.4** Update documentation
  - [ ] Rewrite README.md (quick start)
  - [ ] Add architecture overview
  - [ ] Add API examples
  - [ ] Add troubleshooting section
  - [ ] Add CONTRIBUTING.md
  - **Owner**: TBD
  - **Due**: Week 4
  - **Effort**: 2 hours

### Performance Validation
- [ ] **2.5** Create benchmarks/
  - [ ] bench_sprite_rendering.rs
  - [ ] bench_asset_loading.rs
  - [ ] bench_event_dispatch.rs
  - [ ] Run: `cargo bench --release`
  - [ ] Document results in BENCHMARKS.md
  - **Owner**: TBD
  - **Due**: Week 4
  - **Effort**: 1.5 hours

### Quality Assurance
- [ ] Full test suite: `cargo test --release`
- [ ] Performance verified: 10k sprites @ 60 FPS
- [ ] Examples all runnable
- [ ] Documentation > 80% complete

---

## ✨ Phase 3: ENHANCEMENTS (Weeks 5-8)

### Architecture Extensions
- [ ] **3.1** Scene graph implementation
  - [ ] Create src/core/scene.rs
  - [ ] SceneNode with hierarchy
  - [ ] Transform composition
  - [ ] Component trait
  - [ ] Tests & examples
  - **Owner**: TBD
  - **Due**: Week 5-6
  - **Effort**: 4-5 hours

- [ ] **3.2** Animation system
  - [ ] Create src/core/animation.rs
  - [ ] Keyframe animation
  - [ ] Sprite sheet support
  - [ ] Easing curves
  - [ ] Integration with scene
  - **Owner**: TBD
  - **Due**: Week 6-7
  - **Effort**: 3-4 hours

- [ ] **3.3** Basic physics
  - [ ] Create src/physics/
  - [ ] 2D rigid bodies
  - [ ] Gravity & velocity
  - [ ] Collision callbacks
  - [ ] Simple demo
  - **Owner**: TBD
  - **Due**: Week 7-8
  - **Effort**: 5-6 hours

### Polish & Release
- [ ] **3.4** Final polish
  - [ ] Fix all warnings
  - [ ] Full test coverage
  - [ ] Documentation complete
  - [ ] Performance benchmarks stable
  - [ ] Examples comprehensive
  - **Owner**: TBD
  - **Due**: Week 8
  - **Effort**: 2-3 hours

- [ ] **3.5** v1.0 beta release preparation
  - [ ] Create CHANGELOG.md
  - [ ] Create ROADMAP.md
  - [ ] Create CODE_OF_CONDUCT.md
  - [ ] Setup CI/CD (GitHub Actions)
  - [ ] Tag v1.0-beta
  - **Owner**: TBD
  - **Due**: Week 8
  - **Effort**: 2-3 hours

---

## 📊 Phase 1 Detailed Breakdown

### 1.1 - RenderError Implementation
**File**: `src/render/mod.rs`  
**Status**: ⬜ NOT STARTED  
**Blockers**: None  

Tasks:
```
[ ] Replace struct RenderError with enum
[ ] Add 8+ variants:
    [ ] InitFailed(String)
    [ ] ShaderCompilation(String)
    [ ] OutOfMemory
    [ ] DeviceLost
    [ ] InvalidTexture(String)
    [ ] SurfaceError(String)
    [ ] PipelineSetup(String)
    [ ] Other(String)
[ ] Implement Display via thiserror
[ ] Update wgpu_renderer.rs error conversions
[ ] Add log::error!() calls
[ ] Write test for error messages
[ ] Verify: cargo build && cargo clippy
```

**Review Checklist**:
- [ ] Error enum has good coverage
- [ ] Diagnostic info included
- [ ] thiserror used correctly
- [ ] All error sites updated
- [ ] Tests pass

---

### 1.2 - WgpuRenderer Completion
**File**: `src/render/wgpu_renderer.rs` (649 lines)  
**Status**: ⬜ AUDIT NEEDED  
**Blockers**: Need full file read

Tasks:
```
[ ] Read entire file (already read 150 lines)
[ ] Identify incomplete parts:
    [ ] init() - check device/queue setup
    [ ] surface setup - verify safety
    [ ] render pipeline - confirm exists
    [ ] device.poll() - check synchronization
[ ] Fix all TODO/FIXME comments
[ ] Add error handling (use new RenderError)
[ ] Test on real hardware
[ ] Benchmark basic render
```

**Dependencies**: Completes after 1.1

---

### 1.3 - Input::frame_reset()
**File**: `src/core/events/input.rs`  
**Status**: ⬜ NOT STARTED  
**Blockers**: None

Tasks:
```
[ ] Audit Input struct:
    [ ] Check just_pressed field
    [ ] Check just_released field
    [ ] Check mouse_delta field
[ ] Add frame_reset() method:
    [ ] Clear just_pressed
    [ ] Clear just_released
    [ ] Reset mouse_delta to zero
[ ] Add helper methods:
    [ ] is_key_held(key) -> bool
    [ ] is_key_pressed(key) -> bool
    [ ] is_key_released(key) -> bool
    [ ] any_key_pressed() -> bool
[ ] Update event_handler.rs:
    [ ] Find on_update() method
    [ ] Add frame_reset() call at START
[ ] Write tests:
    [ ] Test just_pressed cleared
    [ ] Test mouse_delta reset
    [ ] Test held keys persist
[ ] Verify: cargo test
```

**Test Code Example**:
```rust
#[test]
fn test_frame_reset() {
    let mut input = Input::new();
    input.just_pressed.insert(Key::A);
    
    input.frame_reset();
    
    assert!(input.just_pressed.is_empty());
}
```

---

### 1.4 - Frame Limiting
**File**: `src/backend/winit_backend.rs`  
**Status**: ⬜ AUDIT NEEDED  
**Blockers**: Need to see full file

Tasks:
```
[ ] Read WinitBackend struct
[ ] Add field: target_frame_time: Option<Duration>
[ ] Add field: last_frame: Instant
[ ] Add method:
    [ ] set_target_fps(fps: u32)
        [ ] Calculate frame_time = 1.0 / fps
        [ ] Store in target_frame_time
[ ] Add method:
    [ ] frame_limiter_sleep()
        [ ] Check target_frame_time
        [ ] Calculate elapsed
        [ ] Sleep if needed
[ ] Find run() method:
    [ ] Add sleep() call after each redraw
[ ] Test:
    [ ] Set target_fps(60)
    [ ] Verify FPS capped
    [ ] Check CPU usage (should be lower)
[ ] Verify: cargo run --release
```

---

### 1.5 - Asset Lifecycle
**File**: `src/core/assets/manager.rs`  
**Status**: ⬜ NOT STARTED  
**Blockers**: Need to read full current impl

Tasks:
```
[ ] Modify AssetManager struct:
    [ ] Add max_memory_bytes: usize
    [ ] Add current_memory_bytes: usize
[ ] Add method:
    [ ] with_limit(bytes: usize) -> Self
[ ] Modify load_image():
    [ ] Calculate image size
    [ ] Check memory limit
    [ ] Return error if over limit
    [ ] Update current_memory_bytes
[ ] Add method:
    [ ] unload_image(id: ImageId) -> bool
        [ ] Remove from map
        [ ] Decrement current_memory_bytes
        [ ] Return success
[ ] Add getter:
    [ ] memory_usage() -> usize
    [ ] memory_limit() -> usize
    [ ] is_over_limit() -> bool
[ ] Add error variant:
    [ ] MemoryExceeded(used, limit)
[ ] Tests:
    [ ] Load until limit
    [ ] Verify error on overflow
    [ ] Unload & verify tracking
    [ ] Clear all & verify zero
```

**Test Code Example**:
```rust
#[test]
fn test_memory_limit() {
    let mut mgr = AssetManager::with_limit(10 * 1024); // 10KB
    
    // Should fail if image > limit
    let result = mgr.load_image("large.png");
    assert!(result.is_err());
}
```

---

## 📌 Dependencies Between Tasks

```
1.1 RenderError
  ↓
1.2 WgpuRenderer (uses new RenderError)
1.3 Input frame_reset
  ↓
Test suite (1.8)
  ├─ Uses fixed Input behavior
  ├─ Uses detailed RenderError
  └─ Tests all fixes

1.4 Frame limiting (independent)
1.5 Asset lifecycle (independent)
1.6 Audio errors (independent)
1.7 Clippy fixes (after all above)

Sequential order: 1.1 → 1.2 → 1.3 → 1.4-1.6 parallel → 1.7 → 1.8
```

---

## 👥 Team Assignment Template

When assigning work, use:

```
### [Task ID] - [Task Name]
**Assigned to**: @username  
**Start Date**: YYYY-MM-DD  
**Due Date**: YYYY-MM-DD  
**Status**: 🟡 In Progress  
**PR**: #[PR number]

Subtasks completed:
- [x] Subtask 1
- [ ] Subtask 2

Blockers: None / Describe

Notes: Any implementation notes
```

---

## 🎯 Weekly Targets

### Week 1:
- [ ] 1.1 RenderError complete
- [ ] 1.2 WgpuRenderer read & understood
- [ ] 1.3 Input::frame_reset complete
- [ ] 1.4 Frame limiting started

**Success Criteria**: 
- [ ] `cargo build` passes
- [ ] `cargo clippy` has <10 warnings
- [ ] `cargo test --lib` passes

### Week 2:
- [ ] 1.2 WgpuRenderer complete
- [ ] 1.5 Asset lifecycle complete
- [ ] 1.7 All clippy warnings fixed
- [ ] 1.8 Unit tests written

**Success Criteria**:
- [ ] Full Phase 1 complete
- [ ] `cargo test` all pass
- [ ] Zero clippy warnings
- [ ] Documentation builds

---

## 📋 Commit Checklist

Before each commit:
```
[ ] cargo fmt (format code)
[ ] cargo clippy (no warnings)
[ ] cargo test (all tests pass)
[ ] cargo build --all-targets (compiles)
[ ] Doc comments added (public API)
[ ] Related tests written
[ ] Commit message clear & descriptive
```

**Commit Message Format**:
```
[Phase 1.X] Feature: Brief description

Detailed explanation of changes.

Related: #issue or #PR numbers
```

---

## 📞 How to Use This Document

1. **For developers**: See "Phase 1 Detailed Breakdown" for exact tasks
2. **For managers**: Check "Weekly Targets" for timeline
3. **For progress**: Update status regularly (⬜→🟡→✅)
4. **For blockers**: Document in "Blockers:" section
5. **For reviews**: Use "Verification" sections

---

## 🚀 How to Mark Tasks Complete

Change status from:
- ⬜ NOT STARTED
- 🟡 IN PROGRESS (when started)
- ✅ COMPLETED (when done + tested + reviewed)

Example:
```
- [x] **1.1** RenderError enum implementation ✅
```

---

**Last Sync**: 2025-12-23  
**Next Sync**: After Phase 1.1 complete  
**Questions**: See QUICK_START.md or ARCHITECTURE_ANALYSIS.md
