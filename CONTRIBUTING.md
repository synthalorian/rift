# Contributing to Rift

First off, thanks for taking the time to contribute! 🎹🦈

Rift is a cross-engine game asset pipeline manager. It converts, validates, and deploys game assets to Unity, Godot, and Unreal — all from a single YAML config.

## Code of Conduct

This project is committed to fostering a welcoming and inclusive environment. Be respectful, constructive, and assume good faith.

## Getting Started

### Prerequisites

- **Rust** 1.75+ (stable) — [install via rustup](https://rustup.rs/)
- **Ruby** 4.0.4+ (for the Rails hub dashboard) — [install via rbenv](https://github.com/rbenv/rbenv)
- **System deps:**
  - Linux: `libsqlite3-dev`, `libfontconfig1-dev`, `ffmpeg`
  - macOS: already available via Xcode CLT
  - Windows: `libsqlite3-dev` via vcpkg or similar

### One-time Setup

```bash
# Clone the repo
git clone https://github.com/synthalorian/rift.git
cd rift

# Build the Rust CLI
cargo build

# (Optional) Set up the Rails dashboard
cd hub
bin/setup
cd ..
```

### Verify It Works

```bash
# Create a test pipeline
mkdir -p /tmp/rift-test/raw
cd /tmp/rift-test
rift init
echo "test" > raw-assets/hello.txt

# Run the pipeline
rift run

# Check status
rift status
```

## Project Structure

```
rift/
├── src/                  # Rust CLI source
│   ├── main.rs           # Entry point
│   ├── cli.rs            # Clap CLI definitions
│   ├── config.rs         # YAML config parser
│   ├── db.rs             # SQLite state tracking
│   ├── api.rs            # Embedded Axum HTTP API
│   ├── error.rs          # Error types
│   ├── pipeline/         # Pipeline execution engine
│   ├── converters/       # Asset converters (texture, model, audio)
│   └── engines/          # Engine exporters (unity, godot, unreal)
├── hub/                  # Rails 8 web dashboard
├── raw-assets/           # Default source asset directory
└── .github/              # CI/CD workflows
```

## Development Workflow

### 1. Pick an Issue

Check the [issues page](https://github.com/synthalorian/rift/issues) for open tasks. Comment to let others know you're working on it.

### 2. Create a Branch

```bash
git checkout -b feat/my-feature
# or
git checkout -b fix/my-bugfix
```

### 3. Make Changes

**Rust CLI changes:**
- Follow existing code conventions (naming, structure, patterns)
- Add tests for new converters, engines, or pipeline steps
- Run `cargo fmt` and `cargo clippy` before committing

**Hub (Rails) changes:**
- Follow Rails 8 conventions (Hotwire, Stimulus, Tailwind)
- Add tests for new controllers/models
- Run `bin/rubocop` before committing

### 4. Test Your Changes

```bash
# Rust tests
cargo test --verbose

# Rust formatting & linting
cargo fmt --check
cargo clippy -- -D warnings

# Hub tests (if you made Rails changes)
cd hub && bin/rails test && cd ..
```

### 5. Commit

Keep commits focused and descriptive:

```bash
git commit -m "feat: add WebP texture support"
git commit -m "fix: handle empty audio files gracefully"
git commit -m "docs: update API endpoint documentation"
```

We use [Conventional Commits](https://www.conventionalcommits.org/) — prefixes like `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:` help with changelog generation.

### 6. Submit a Pull Request

- Push your branch and open a PR against `main`
- Fill out the PR template (checklist, description, related issue)
- CI will run automatically — make sure all checks pass
- Request review from a maintainer

## Coding Standards

### Rust

- **Formatting:** `cargo fmt` (rustfmt with default settings)
- **Linting:** `cargo clippy` — address all warnings
- **Naming:** `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants
- **Error handling:** Use `anyhow::Result` for CLI code, `thiserror` for library errors
- **Tests:** Unit tests in `#[cfg(test)] mod tests` at the bottom of each module file
- **Documentation:** Document public APIs with `///` doc comments

### Ruby / Rails (Hub)

- **Style:** `rubocop` with `rubocop-rails-omakase` rules
- **Naming:** `snake_case` for methods/variables, `PascalCase` for classes, `SCREAMING_SNAKE_CASE` for constants
- **Views:** Use Hotwire (Turbo + Stimulus) for interactivity
- **Styling:** Tailwind CSS with the synthwave84 theme
- **Tests:** Minitest with fixtures

## Adding a New Converter

1. Create `src/converters/new_type.rs`
2. Implement the converter logic (see `texture.rs` or `audio.rs` for reference)
3. Add a file pattern rule to the pipeline config
4. Register the converter in `src/converters/mod.rs`
5. Add tests for valid/invalid inputs
6. Run `cargo test` to verify

## Adding a New Engine Exporter

1. Create `src/engines/new_engine.rs`
2. Implement the `EngineExporter` trait:
   - `fn export_texture(&self, ...) -> Result<()>`
   - `fn export_audio(&self, ...) -> Result<()>`
   - `fn engine_name(&self) -> &str`
3. Register in `src/engines/mod.rs`
4. Update the config parser to accept the new engine key
5. Add tests

## Release Process

Maintainers handle releases:

1. Update version in `Cargo.toml`
2. Commit: `chore: bump version to x.y.z`
3. Tag: `git tag vx.y.z && git push --tags`
4. CI builds binaries for Linux, macOS, and Windows
5. Draft release notes on GitHub

## Getting Help

- Open a [discussion](https://github.com/synthalorian/rift/discussions) for questions
- File an [issue](https://github.com/synthalorian/rift/issues) for bugs or feature requests
- Check existing issues before opening a new one

---

**Rift. Bridge the engines.** 🎹🦈
