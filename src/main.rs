use clap::CommandFactory;
use clap::Parser;
use colored::Colorize;
use std::path::Path;
use std::result::Result as StdResult;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> StdResult<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = rift::cli::Cli::parse();

    match cli.command {
        rift::cli::Commands::Init { engine } => {
            cmd_init(&engine)?;
        }
        rift::cli::Commands::Run {
            config,
            clean,
            verbose,
        } => {
            if verbose {
                std::env::set_var("RUST_LOG", "debug");
            }
            cmd_run(config.as_deref(), clean).await?;
        }
        rift::cli::Commands::Watch {
            config,
            port,
            dashboard,
        } => {
            cmd_watch(config.as_deref(), port, dashboard).await?;
        }
        rift::cli::Commands::Status { db, assets } => {
            cmd_status(db.as_deref(), assets)?;
        }
        rift::cli::Commands::Completions { shell } => {
            cmd_completions(shell)?;
        }
        rift::cli::Commands::Upgrade {} => {
            cmd_upgrade().await?;
        }
    }

    Ok(())
}

fn cmd_init(engine: &str) -> StdResult<(), Box<dyn std::error::Error>> {
    let path = Path::new("rift.yml");
    if path.exists() {
        eprintln!(
            "{}",
            "⚠️  rift.yml already exists in this directory"
                .yellow()
                .bold()
        );
        eprintln!("   Remove it first: {}", "rm rift.yml".bright_cyan());
        return Err("rift.yml already exists".into());
    }

    let (source_root, targets) = match engine {
        "unity" => (
            "raw-assets",
            vec![serde_json::json!({
                "engine": "unity",
                "project": "Assets/Rift",
                "textures": {"format": "png", "max_width": 2048, "max_height": 2048},
                "audio": {"format": "ogg", "bitrate": "128k"}
            })],
        ),
        "godot" => (
            "raw-assets",
            vec![serde_json::json!({
                "engine": "godot",
                "project": "assets/rift",
                "textures": {"format": "png", "max_width": 2048, "max_height": 2048},
                "audio": {"format": "ogg", "bitrate": "128k"}
            })],
        ),
        "unreal" => (
            "raw-assets",
            vec![serde_json::json!({
                "engine": "unreal",
                "project": "Content/Rift",
                "textures": {"format": "png", "max_width": 4096, "max_height": 4096},
                "audio": {"format": "wav", "bitrate": "44100"}
            })],
        ),
        _ => (
            "raw-assets",
            vec![
                serde_json::json!({
                    "engine": "unity",
                    "project": "unity-project/Assets/Rift",
                    "textures": {"format": "png", "max_width": 2048, "max_height": 2048},
                    "audio": {"format": "ogg", "bitrate": "128k"}
                }),
                serde_json::json!({
                    "engine": "godot",
                    "project": "godot-project/assets/rift",
                    "textures": {"format": "png", "max_width": 1024, "max_height": 1024},
                    "audio": {"format": "ogg", "bitrate": "128k"}
                }),
                serde_json::json!({
                    "engine": "unreal",
                    "project": "unreal-project/Content/Rift",
                    "textures": {"format": "png", "max_width": 4096, "max_height": 4096},
                    "audio": {"format": "wav", "bitrate": "44100"}
                }),
            ],
        ),
    };

    let config = serde_json::json!({
        "pipeline": {
            "name": "game-assets",
            "version": 1
        },
        "source": {
            "root": source_root,
            "watch": true
        },
        "targets": targets,
        "hooks": {
            "pre_run": null,
            "post_run": null,
            "on_error": null
        },
        "rules": [
            {
                "pattern": "**/*.psd",
                "convert": "textures",
                "remove_original": false
            },
            {
                "pattern": "**/*.png",
                "convert": "textures",
                "validate": false
            },
            {
                "pattern": "**/*.jpg",
                "convert": "textures"
            },
            {
                "pattern": "**/*.jpeg",
                "convert": "textures"
            },
            {
                "pattern": "**/*.tga",
                "convert": "textures"
            },
            {
                "pattern": "**/*.wav",
                "convert": "audio"
            },
            {
                "pattern": "**/*.mp3",
                "convert": "audio"
            },
            {
                "pattern": "**/*.fbx",
                "convert": "models",
                "validate": true
            },
            {
                "pattern": "**/*.gltf",
                "convert": "models",
                "validate": true
            },
            {
                "pattern": "**/*.glb",
                "convert": "models",
                "validate": true
            },
            {
                "pattern": "**/*.obj",
                "convert": "models",
                "validate": true
            }
        ]
    });

    let content = serde_yaml::to_string(&config)?;
    std::fs::write(path, &content)?;

    // Create source root and state directory
    std::fs::create_dir_all(source_root)?;
    std::fs::create_dir_all(".rift")?;

    println!(
        "{}",
        "✅ Initialized rift pipeline in ./rift.yml".green().bold()
    );
    println!("   {}  ./{}", "Source:".bold(), source_root);
    println!("   {}   .rift/", "State:".bold());
    println!();
    println!(
        "   {}",
        "Run `rift run` to process existing assets".bright_cyan()
    );

    Ok(())
}

async fn cmd_run(
    config_path: Option<&str>,
    clean: bool,
) -> StdResult<(), Box<dyn std::error::Error>> {
    let config = match config_path {
        Some(path) => rift::config::PipelineConfig::from_file(Path::new(path))?,
        None => rift::config::PipelineConfig::discover()?,
    };

    let db_path = get_db_path()?;
    let db = rift::db::AssetDb::open(&db_path)?;
    let runner = rift::pipeline::Runner::new(db);

    let results = runner.run_once(&config, clean)?;

    let success = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    println!(
        "\n{} Pipeline complete: {} converted, {} errors",
        "📊".bright_cyan(),
        format!("{}", success).green(),
        format!("{}", failed).red()
    );

    for result in &results {
        if result.success {
            println!("   {} {}", "✓".green().bold(), result.relative_path.cyan());
            for out in &result.output_paths {
                println!(
                    "     {} {}",
                    "→".bright_blue(),
                    out.display().to_string().dimmed()
                );
            }
        } else {
            println!(
                "   {} {} — {}",
                "✗".red().bold(),
                result.relative_path.cyan(),
                result.error.as_deref().unwrap_or("unknown").red()
            );
        }
    }

    Ok(())
}

async fn cmd_watch(
    config_path: Option<&str>,
    port: u16,
    dashboard: bool,
) -> StdResult<(), Box<dyn std::error::Error>> {
    let config = match config_path {
        Some(path) => rift::config::PipelineConfig::from_file(Path::new(path))?,
        None => rift::config::PipelineConfig::discover()?,
    };

    let db_path = get_db_path()?;
    let db = rift::db::AssetDb::open(&db_path)?;

    if dashboard {
        let api_db = rift::db::AssetDb::open(&db_path)?;
        tokio::spawn(async move {
            if let Err(e) = rift::api::serve(api_db, port).await {
                tracing::error!("API server error: {}", e);
            }
        });
        println!("📡 Embedded dashboard: http://localhost:{}", port);
    }

    let runner = rift::pipeline::Runner::new(db);
    let watcher = rift::pipeline::Watcher::new(runner);
    watcher.watch(&config)?;

    Ok(())
}

fn cmd_status(
    db_path: Option<&str>,
    show_assets: bool,
) -> StdResult<(), Box<dyn std::error::Error>> {
    let path = match db_path {
        Some(p) => Path::new(p).to_path_buf(),
        None => {
            let cwd = std::env::current_dir()?;
            cwd.join(".rift").join("state.db")
        }
    };

    if !path.exists() {
        return Err("No rift database found. Run `rift run` first.".into());
    }

    let db = rift::db::AssetDb::open(&path)?;

    let counts = db.get_asset_counts()?;
    println!(
        "{} {}\n",
        "📦".bright_cyan(),
        format!("Asset Database: {}", path.display()).bright_white()
    );

    let total: u32 = counts.values().sum();
    let ok = counts.get("ok").copied().unwrap_or(0);
    let pending = counts.get("pending").copied().unwrap_or(0);
    let error_count = counts.get("error").copied().unwrap_or(0);

    println!("   {}  {}", "Total:".bold(), total);
    println!("   {}   {}", "OK:".green().bold(), ok);
    println!("   {}  {}", "Pending:".yellow().bold(), pending);
    println!("   {}  {}\n", "Errors:".red().bold(), error_count);

    let runs = db.get_recent_runs(5)?;
    if !runs.is_empty() {
        println!("{}", "📋 Recent Pipeline Runs:".bright_white());
        for run in &runs {
            let status_color = match run.status.as_str() {
                "completed" => "completed".green(),
                "running" => "running".cyan(),
                "failed" => "failed".red(),
                _ => run.status.normal(),
            };
            println!(
                "   {} [{}] {} converted, {} errors",
                run.id[..8].dimmed(),
                status_color,
                run.converted,
                run.errors
            );
        }
    }

    if show_assets {
        println!();
        println!("{}", "📄 Assets:".bright_white());
        let assets = db.get_assets(None, 50, 0)?;
        for asset in &assets {
            let (icon, color) = match asset.status.as_str() {
                "ok" => ("✓", "green"),
                "error" => ("✗", "red"),
                _ => ("⋯", "yellow"),
            };
            let colored_status = match color {
                "green" => asset.status.green().to_string(),
                "red" => asset.status.red().to_string(),
                _ => asset.status.yellow().to_string(),
            };
            let colored_icon = match color {
                "green" => icon.green().bold().to_string(),
                "red" => icon.red().bold().to_string(),
                _ => icon.yellow().to_string(),
            };
            println!(
                "   {} {} — {}",
                colored_icon,
                asset.relative_path.cyan(),
                colored_status
            );
        }
    }

    Ok(())
}

async fn cmd_upgrade() -> StdResult<(), Box<dyn std::error::Error>> {
    println!("{}", "🔍 Checking for updates...".bright_cyan().bold());

    let client = reqwest::Client::builder()
        .user_agent("rift-upgrade/0.1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let url = "https://api.github.com/repos/synthalorian/rift/releases/latest";
    let resp = client.get(url).send().await?;

    if !resp.status().is_success() {
        println!("   {}", "Could not check for updates.".yellow());
        println!("   {}", "Manual update: cargo install --path .".dimmed());
        return Ok(());
    }

    let release: serde_json::Value = resp.json().await?;
    let latest_tag = release["tag_name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let current_version = env!("CARGO_PKG_VERSION");

    println!("   {}  v{}", "Current version:".bold(), current_version);
    println!(
        "   {}  {}",
        "Latest version:".bold(),
        latest_tag.bright_cyan()
    );

    if latest_tag.trim_start_matches('v') == current_version {
        println!("\n{}  You're on the latest version!", "✅".green().bold());
    } else {
        println!("\n{}  Update available!", "📦".bright_cyan().bold());
        println!(
            "   Run: {}",
            "cargo install --git https://github.com/synthalorian/rift.git".bright_white()
        );
        println!(
            "   Or download from: {}",
            "https://github.com/synthalorian/rift/releases".bright_cyan()
        );
    }

    Ok(())
}

fn cmd_completions(shell: clap_complete::Shell) -> StdResult<(), Box<dyn std::error::Error>> {
    let mut cmd = rift::cli::Cli::command();
    rift::cli::print_completions(shell, &mut cmd);
    Ok(())
}

fn get_db_path() -> StdResult<std::path::PathBuf, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()?;
    let db_dir = cwd.join(".rift");
    std::fs::create_dir_all(&db_dir)?;
    Ok(db_dir.join("state.db"))
}
