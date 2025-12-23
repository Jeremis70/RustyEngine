# RustyEngine - Configuration & Setup Recommendations

## 📁 Recommended Folder Structure

```
RustyEngine/
├── src/                          # Source code
│   ├── main.rs
│   ├── lib.rs                    # ✨ NEW: Export public API
│   ├── audio/
│   ├── backend/
│   ├── core/
│   │   ├── animation.rs          # 📦 Phase 3
│   │   ├── scene.rs              # 📦 Phase 3
│   │   └── ...
│   ├── game/
│   ├── graphics/
│   ├── math/
│   ├── physics/                  # 📦 Phase 3 (new folder)
│   └── render/
│
├── tests/                        # Integration tests
│   ├── integration_tests.rs      # 📦 Phase 2
│   ├── common/                   # Shared test utilities
│   │   └── mod.rs
│   └── fixtures/                 # Test data
│       └── test.png
│
├── benches/                      # Performance benchmarks
│   ├── sprite_rendering.rs       # 📦 Phase 2
│   ├── asset_loading.rs          # 📦 Phase 2
│   └── common.rs
│
├── examples/                     # Runnable examples
│   ├── sprite_demo.rs            # 📦 Phase 2
│   ├── audio_demo.rs             # 📦 Phase 2
│   ├── input_demo.rs             # 📦 Phase 2
│   └── shapes_demo.rs            # 📦 Phase 2
│
├── docs/                         # Documentation
│   ├── ARCHITECTURE.md           # ✅ CREATED
│   ├── API.md                    # 📦 Phase 2
│   └── TUTORIAL.md               # 📦 Phase 3
│
├── .github/
│   └── workflows/
│       ├── ci.yml                # 📦 CI/CD Pipeline
│       └── publish.yml           # 📦 Publish to crates.io
│
├── Cargo.toml                    # ✅ Project manifest
├── Cargo.lock                    # Lock file (commit)
├── .gitignore                    # ✅ Update
├── .rustfmt.toml                 # Format config
├── .clippy.toml                  # Linting config
│
├── README.md                     # ✅ Top-level info
├── CONTRIBUTING.md               # 📦 Phase 2
├── CODE_OF_CONDUCT.md            # 📦 Phase 2
├── LICENSE                       # MIT/Apache-2.0
│
├── ARCHITECTURE_ANALYSIS.md      # ✅ This analysis
├── IMPROVEMENT_PLAN.md           # ✅ Detailed plan
├── COMPARISON_ANALYSIS.md        # ✅ Competitive analysis
├── EXECUTIVE_SUMMARY.md          # ✅ 1-page summary
├── QUICK_START.md                # ✅ Implementation guide
├── TODO.md                       # ✅ Task tracking
└── BENCHMARKS.md                 # 📦 Results file

✅ = Already exists or created
📦 = To be added
```

---

## ⚙️ Cargo.toml Recommendations

```toml
[package]
name = "rusty-engine"  # Change from RustyEngine
version = "0.1.0"
edition = "2024"  # ⚠️ VERIFY: Should be 2021
authors = ["Your Team <team@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourorg/rusty-engine"
documentation = "https://docs.rs/rusty-engine"
description = "A modern, type-safe 2D game engine"
keywords = ["game", "engine", "2d", "graphics", "wgpu"]
categories = ["game-engines", "graphics"]

[dependencies]
wgpu = "27.0.1"
winit = "0.30.12"
thiserror = "2.0.17"
log = "0.4.29"
env_logger = "0.11.8"
pollster = "0.4.0"
raw-window-handle = "0.6.2"
bytemuck = { version = "1.20", features = ["derive"] }
rodio = "0.17.1"
image = { version = "0.25", default-features = false, features = ["png", "jpeg", "bmp"] }

[dev-dependencies]
criterion = "0.5"  # For benchmarks
tempfile = "3.8"   # For test fixtures

[[bench]]
name = "sprite_rendering"
harness = false  # Use criterion

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.bench]
inherits = "release"
```

---

## 🚀 GitHub Actions CI/CD

### File: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  # Test on multiple platforms
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: ${{ matrix.rust }}
          override: true
      
      - uses: actions-rs/cargo@v1
        with:
          command: test
          args: --verbose

  # Linting with clippy
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          components: clippy
      
      - uses: actions-rs/clippy-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          args: -- -D warnings

  # Code formatting check
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          components: rustfmt
      - run: cargo fmt -- --check

  # Documentation builds
  docs:
    name: Docs
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      
      - name: Build docs
        env:
          RUSTDOCFLAGS: -D warnings
        run: cargo doc --no-deps --all-features

  # Security audit
  security_audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check-action@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### File: `.github/workflows/publish.yml`

```yaml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    name: Publish
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      
      - name: Publish
        run: cargo publish --token ${{ secrets.CARGO_TOKEN }}
```

---

## 🔧 Local Development Setup

### File: `.rustfmt.toml`

```toml
# Format rules
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
comments_width = 80
wrap_comments = true
comment_width = 80
normalize_comments = true
normalize_doc_attributes = true
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
use_small_heuristics = "Default"
format_code_in_doc_comments = true
merge_derives = true
use_try_shorthand = true
use_field_init_shorthand = true
force_explicit_abi = true
```

### File: `.clippy.toml`

```toml
# Clippy linting configuration
too-many-arguments-threshold = 8
type-complexity-threshold = 500
single-char-binding-names-threshold = 5
```

### File: `.gitignore` (Update)

```
# Rust
target/
Cargo.lock
**/*.rs.bk
*.pdb

# IDEs
.vscode/
.idea/
*.swp
*.swo
*~
.DS_Store

# OS
.DS_Store
Thumbs.db

# Test artifacts
*.profdata

# Benchmark artifacts
benches/target/
```

---

## 🎯 VSCode Configuration

### File: `.vscode/settings.json`

```json
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
  "editor.rulers": [80, 100, 120],
  "files.exclude": {
    "**/target": true
  }
}
```

### File: `.vscode/extensions.json`

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "tamasfe.even-better-toml",
    "serayuzgur.crates"
  ]
}
```

---

## 📊 Cargo Commands Cheatsheet

```bash
# Development
cargo build           # Debug build
cargo run             # Run main.rs
cargo check           # Fast syntax check
cargo clippy          # Linting (strict)
cargo fmt             # Auto format

# Testing
cargo test            # All tests
cargo test --lib     # Unit tests only
cargo test --test '*' # Integration tests
cargo test -- --nocapture  # Show output
cargo test -- --test-threads=1  # Sequential

# Benchmarking
cargo bench --release # Run all benchmarks
cargo bench sprite_rendering  # Specific bench

# Documentation
cargo doc --open      # Generate & open docs
cargo doc --no-deps   # Without dependencies

# Quality
cargo audit           # Security check
cargo outdated        # Check dependency versions

# Release
cargo build --release # Optimized build
cargo publish --dry-run  # Test publish
cargo publish         # Actually publish
```

---

## 📋 Code Review Checklist

Before merging PRs, verify:

**Code Quality**:
- [ ] `cargo fmt` applied
- [ ] `cargo clippy -- -D warnings` passes
- [ ] No unsafe blocks (or reviewed)
- [ ] No unwrap()/panic! (use Result<>)
- [ ] Comments on complex logic
- [ ] No debug prints (use log crate)

**Testing**:
- [ ] New features have tests
- [ ] All tests pass (`cargo test`)
- [ ] >70% code coverage
- [ ] Edge cases covered

**Documentation**:
- [ ] Public API documented
- [ ] Examples provided (if user-facing)
- [ ] CHANGELOG.md updated
- [ ] README updated if needed

**Performance**:
- [ ] No performance regression
- [ ] Benchmarks run (if applicable)
- [ ] No unnecessary allocations
- [ ] Comments on hot paths

---

## 🎓 Development Workflow

### 1. Start Feature
```bash
git checkout -b feature/my-feature
cargo build
cargo test
```

### 2. Develop & Test
```bash
cargo clippy -- -D warnings
cargo fmt
cargo test --all
```

### 3. Benchmark (if perf-related)
```bash
cargo bench --release
```

### 4. Document
```bash
cargo doc --no-deps --open
# Review generated docs
```

### 5. Commit
```bash
git add .
git commit -m "feat: description"
```

### 6. Create PR
- Describe changes
- Link related issues
- Wait for CI to pass
- Address review comments

### 7. Merge
```bash
git checkout main
git pull origin main
git merge feature/my-feature
git push origin main
```

---

## 🚨 Important Principles

1. **No direct commits to main** - Always use branches + PRs
2. **CI must pass** - All checks before merge
3. **Tests required** - New code = new tests
4. **Documentation** - Public API must be documented
5. **Performance** - Benchmark before/after for perf changes
6. **Security** - Run `cargo audit` regularly
7. **Code review** - At least 1 review per PR

---

## 🔒 Security Practices

```bash
# Before releases
cargo audit              # Check vulnerabilities
cargo outdated          # Check dependencies
cargo test --release    # Full test suite
cargo bench --release   # Performance check

# In Cargo.toml
[dependencies]
# Always specify versions, avoid wildcards
wgpu = "27.0.1"        # ✅ Explicit version
thiserror = "2.0.17"   # ✅ Explicit version

# Avoid
some-lib = "*"         # ❌ Too loose
other-lib = ">=1.0"    # ❌ Too loose
```

---

## 📈 Metrics to Track

Create `BENCHMARKS.md` to track:

```markdown
# Performance Benchmarks

## Sprite Rendering (10k sprites)
| Date | FPS | Time/Frame | GPU | Notes |
|------|-----|-----------|-----|-------|
| 2025-12-23 | 60 | 16.7ms | N/A | Baseline (no batching) |
| 2026-01-15 | 200+ | 5ms | GTX1080 | After batching |

## Asset Loading
| Test | Before | After | Improvement |
|------|--------|-------|-------------|
| Load 100 PNGs | 2.5s | 0.8s | 3.1x faster |
| Memory (100 images) | 150MB | 80MB | -46% |

## Test Coverage
- Unit tests: 75%
- Integration tests: 60%
- Overall: 70%
```

---

## 🎯 Success Metrics

### Phase 1 Done:
- ✅ `cargo build --all-targets` clean
- ✅ `cargo test` all pass
- ✅ `cargo clippy -- -D warnings` clean
- ✅ `cargo fmt --check` clean
- ✅ Doc coverage: >80%

### Phase 2 Done:
- ✅ 10k sprites @ 60 FPS
- ✅ <16ms per frame
- ✅ Examples: 4+ runnable
- ✅ Tests: integration + benchmarks
- ✅ README: Complete

### v1.0 Beta:
- ✅ All above met
- ✅ Scene graph optional
- ✅ Animation system optional
- ✅ Community feedback positive
- ✅ No critical bugs found

---

## 📞 References

- [Rust Book](https://doc.rust-lang.org/book/)
- [wgpu Docs](https://docs.rs/wgpu/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Rustfmt Options](https://rust-lang.github.io/rustfmt/)

---

**Last Updated**: 2025-12-23  
**Next Review**: After Phase 1 complete
