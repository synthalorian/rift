# Rift — Game Asset Pipeline Manager

**Goal:** Build a cross-engine game asset pipeline toolchain — CLI watcher with Rust, web dashboard with Rails — that automatically converts, validates, and deploys game assets to Unity, Unreal, and Godot projects.

**Architecture:** A Rust binary (`rift`) that reads pipeline definitions from YAML, watches asset source directories, converts/validates files, and outputs to engine-specific project directories with the correct import metadata. A Rails 8 web app (`hub/`) provides a visual dashboard for pipeline runs, error logs, and asset browsing. The Rust binary embeds a small HTTP server for the Rails app to query.

**Tech Stack:**
- **Rust CLI:** clap (derive), notify (file watcher), serde/yaml (pipeline config), image (texture conversion), walkdir (asset traversal), rusqlite (state tracking), sha2 (content hashing), axum (embedded HTTP API)
- **Rails 8 Hub:** Rails 8 with SQLite, Hotwire (Turbo + Stimulus), Tailwind CSS (synthwave84 theme), Propshaft

---

## Project Structure

```
rift/
├── src/
│   ├── main.rs              # Entry point, CLI dispatch
│   ├── lib.rs               # Core library exports
│   ├── cli.rs               # Clap CLI definition
│   ├── config.rs            # Pipeline YAML config parsing
│   ├── pipeline/            # Pipeline execution engine
│   │   ├── mod.rs
│   │   ├── runner.rs        # Step-by-step pipeline execution
│   │   └── step.rs          # Individual pipeline step types
│   ├── watcher.rs           # File system watcher (notify-based)
│   ├── converters/          # Asset conversion modules
│   │   ├── mod.rs
│   │   ├── texture.rs       # Image resize, format conversion
│   │   ├── model.rs         # Model validation (FBX/glTF checks)
│   │   └── audio.rs         # Audio compression & validation
│   ├── engines/             # Engine-specific exporters
│   │   ├── mod.rs
│   │   ├── unity.rs         # Unity .meta generation
│   │   ├── unreal.rs        # Unreal import settings
│   │   └── godot.rs         # Godot .import files
│   ├── db.rs                # SQLite state tracking
│   ├── api.rs               # Embedded Axum HTTP API
│   └── error.rs             # Error types
├── hub/                     # Rails 8 web dashboard
│   ├── app/
│   │   ├── controllers/
│   │   ├── models/
│   │   ├── views/
│   │   ├── javascript/
│   │   └── ...
│   └── ...
├── Cargo.toml
├── Gemfile
└── rift.yml.example         # Example pipeline config
```

---

## Pipeline YAML Format (MVP)

```yaml
pipeline:
  name: "game-assets"
  version: 1

source:
  root: "~/Projects/raw-assets"
  watch: true       # Watch for changes

targets:
  - engine: unity
    project: "unity-project/Assets/Rift"
    textures:
      format: png
      max_width: 2048
      max_height: 2048
    audio:
      format: ogg
      bitrate: 128k
  - engine: godot
    project: "godot-project/assets/rift"
    textures:
      format: png
      max_width: 1024
  - engine: unreal
    project: "unreal-project/Content/Rift"

rules:
  - pattern: "**/*.psd"
    convert: textures
    remove_original: false
  - pattern: "**/*.wav"
    convert: audio
  - pattern: "**/*.fbx"
    validate: true
```

---

## API Surface (Rust → Rails)

The Rust daemon exposes a local HTTP API on `localhost:8910`:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/status` | GET | Daemon health + pipeline state |
| `/api/v1/assets` | GET | All tracked assets (paginated) |
| `/api/v1/assets/:hash` | GET | Asset details + conversion history |
| `/api/v1/pipeline-runs` | GET | Recent pipeline execution logs |
| `/api/v1/errors` | GET | Failed conversions + validation errors |
| `/api/v1/pipeline/run` | POST | Trigger a full pipeline run |

---

## Implementation Order

### Phase 1: Rust CLI Core
1. Scaffold Cargo project with deps
2. CLI dispatch (rift init, rift run, rift watch, rift status)
3. Pipeline YAML config parser
4. Asset traversal and content hashing
5. SQLite state database (tracks asset hash, conversions, errors)
6. Texture converter (resize, format conversion via `image`)
7. Engine exporters (Unity .meta, Godot .import)
8. File watcher with debounce
9. Pipeline runner (orchestrates convert → validate → export)
10. Embedded Axum API

### Phase 2: Rails Dashboard
1. Scaffold Rails 8 app in hub/
2. Sync model (mirrors SQLite state via API)
3. Pipeline runs view (timeline of conversions)
4. Asset browser (search, filter by engine, status)
5. Error log view
6. Pipeline config editor (via YAML textarea)
7. Synthwave84 theme

---

## Verification

```bash
# Create test assets
mkdir -p /tmp/rift-test/raw
mkdir -p /tmp/rift-test/unity-project/Assets/Rift
mkdir -p /tmp/rift-test/godot-project/assets/rift

# Create a fake PSD
convert -size 100x100 xc:red /tmp/rift-test/raw/test.psd

# Run pipeline
cd rift && cargo run -- run --config /tmp/rift-test/rift.yml

# Verify output
ls /tmp/rift-test/unity-project/Assets/Rift/test.png
ls /tmp/rift-test/godot-project/assets/rift/test.png
ls /tmp/rift-test/unity-project/Assets/Rift/test.png.meta

# Check status
cargo run -- status
```
