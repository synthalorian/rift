# Rift

**Cross-engine game asset pipeline manager.**

[![CI](https://github.com/synthalorian/rift/actions/workflows/ci.yml/badge.svg)](https://github.com/synthalorian/rift/actions/workflows/ci.yml)

Drop raw assets in one directory. Rift converts textures, validates models, compresses audio, and deploys to Unity, Godot, and Unreal — all from a single YAML config.

```
               raw-assets/              rift.yml
                    │                      │
                    ▼                      ▼
             ┌──────────────┐     ┌──────────────┐
             │  rift watch  │────▶│  Pipeline    │
             │  or rift run │     │  Engine      │
             └──────────────┘     └──────┬───────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    ▼                    ▼                    ▼
           unity-project/       godot-project/        unreal-project/
           Assets/Rift          assets/rift           Content/Rift
           (Unity .meta)        (Godot .import)       (Unreal JSON)
```

## Quick Start

```bash
# Install
git clone https://github.com/synthalorian/rift.git
cd rift && cargo install --path .

# Initialize a pipeline
mkdir my-game && cd my-game
rift init

# Drop assets in raw-assets/ and run
rift run

# Or watch for changes
rift watch

# Check status
rift status --assets
```

## Pipeline YAML

The heart of Rift is a YAML config file. Here's a real example from `rift.yml.example`:

```yaml
pipeline:
  name: "game-assets"
  version: 1

source:
  root: "raw-assets"
  watch: true

targets:
  - engine: unity
    project: "unity-project/Assets/Rift"
    textures:
      format: png
      max_width: 2048
      max_height: 2048
    audio:
      format: ogg
      bitrate: "128k"

  - engine: godot
    project: "godot-project/assets/rift"
    textures:
      format: png
      max_width: 1024
      max_height: 1024

  - engine: unreal
    project: "unreal-project/Content/Rift"
    textures:
      format: png
      max_width: 4096
      max_height: 4096

rules:
  - pattern: "**/*.{png,jpg,jpeg,tga,bmp,webp,tiff}"
    convert: textures
  - pattern: "**/*.{wav,mp3,aiff,flac}"
    convert: audio
  - pattern: "**/*.{fbx,gltf,glb,obj,blend}"
    convert: models
    validate: true

hooks:
  pre_run: "echo 'Pipeline starting...'"
  post_run: "notify-send 'Pipeline complete'"
  on_error: "echo 'Pipeline failed!'"
```

## Commands

| Command | Description |
|---------|-------------|
| `rift init` | Generate a `rift.yml` in the current directory |
| `rift run` | Scan source, convert/validate/export once |
| `rift run --clean` | Force re-conversion of all assets (skip cache) |
| `rift watch` | Watch source directory, auto-run on changes |
| `rift status` | Show database stats and recent pipeline runs |
| `rift status --assets` | Show per-asset status |

### `rift init [--engine unity|godot|unreal|all]`

Generates a `rift.yml` pre-configured for one or all engines. Creates the `raw-assets/` directory and `.rift/` for state.

### `rift run`

Scans every file in the source root, matches it against rules, and:
- **Textures** — resizes to fit max dimensions, converts format, generates engine metadata
- **Audio** — converts WAV to OGG (or copies), validates sample rate
- **Models** — validates FBX/glTF/OBJ headers, checks for common issues, copies to target

### `rift watch`

Same as `run`, but stays running and watches for filesystem changes using `notify`. Automatically triggers a pipeline run when new assets appear or existing ones change.

### `rift status`

Queries the SQLite database in `.rift/state.db` for:
- Asset counts by status (ok / pending / error)
- Recent pipeline runs
- Per-asset conversion history (with `--assets`)

## Dashboard

Rift includes an embedded HTTP API (port 8910) for the Rails dashboard:

```bash
rift watch --dashboard
# API: http://localhost:8910/api/v1/status
```

## Architecture

```
rift/
├── src/
│   ├── main.rs              Entry point, CLI dispatch
│   ├── lib.rs               Library exports
│   ├── cli.rs               Clap CLI definition
│   ├── config.rs            Pipeline YAML parsing
│   ├── pipeline/            Pipeline execution engine
│   │   └── mod.rs           Runner + Watcher
│   ├── converters/          Asset conversion modules
│   │   ├── texture.rs       Image resize + format conversion
│   │   ├── model.rs         FBX/glTF/OBJ validation
│   │   └── audio.rs         WAV parsing + OGG conversion
│   ├── engines/             Engine-specific exporters
│   │   ├── unity.rs         .meta file generation
│   │   ├── godot.rs         .import file generation
│   │   └── unreal.rs        JSON import settings
│   ├── db.rs                SQLite state tracking
│   ├── api.rs               Embedded Axum HTTP API
│   └── error.rs             Error types
├── hub/                     Rails 8 web dashboard
├── rift.yml.example         Example pipeline config
├── PLAN.md                  Architecture & implementation plan
└── Cargo.toml
```

## Requirements

- **Rust** 1.75+ (stable)
- System deps: `libsqlite3-dev`, `libfontconfig` (for image crate)

## Tech Stack

| Component | Language | Purpose |
|-----------|----------|---------|
| CLI + Engine | Rust | Pipeline execution, file watching, conversion |
| Web Dashboard | Ruby on Rails 8 | Pipeline visualizer, asset browser, error viewer |
| State | SQLite | Asset cache, conversion history, error log |
| API | Axum (embedded) | Interface for Rails dashboard |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, coding standards, and the development workflow.

## License

[Apache-2.0](LICENSE)

---

**Rift. Bridge the engines.**
