use crate::converters;
use crate::db::AssetDb;
use crate::engines;
use crate::RiftError;
use glob::Pattern;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

/// A pending asset job — file info extracted ahead of conversion
struct AssetJob {
    path: PathBuf,
    relative: String,
    rule_index: usize,
}

/// Result of processing a single asset
#[derive(Debug)]
pub struct AssetResult {
    pub relative_path: String,
    pub success: bool,
    pub output_paths: Vec<PathBuf>,
    pub error: Option<String>,
}

/// Runs the pipeline: traverse assets, match rules, convert/validate/export
pub struct Runner {
    db: AssetDb,
}

// process_asset is a free function (no &self) so it can run in parallel
fn process_asset(
    path: &Path,
    relative: &str,
    rule: &crate::config::RuleConfig,
    config: &crate::config::PipelineConfig,
) -> AssetResult {
    let convert_type = rule.convert.as_deref().unwrap_or("copy");

    let start = Instant::now();

    let result = (|| -> crate::Result<AssetResult> {
        let mut output_paths = Vec::new();

        match convert_type {
            "textures" => {
                for target in &config.targets {
                    let out = converters::texture::convert(path, target)?;
                    engines::generate_meta(&out, target)?;
                    output_paths.push(out);
                }
            }
            "audio" => {
                for target in &config.targets {
                    let out = converters::audio::convert(path, target)?;
                    output_paths.push(out);
                }
            }
            "models" => {
                for target in &config.targets {
                    let target_dir = target.project.join(relative);
                    if let Some(parent) = target_dir.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    if rule.validate {
                        converters::model::validate(path)?;
                    }
                    std::fs::copy(path, &target_dir)?;
                    output_paths.push(target_dir);
                }
            }
            _ => {
                for target in &config.targets {
                    let target_path = target.project.join(relative);
                    if let Some(parent) = target_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(path, &target_path)?;
                    output_paths.push(target_path);
                }
            }
        }

        Ok(AssetResult {
            relative_path: relative.to_string(),
            success: true,
            output_paths,
            error: None,
        })
    })();

    let elapsed = start.elapsed();

    match result {
        Ok(r) => {
            info!("  ✓ {} ({:?})", relative, elapsed);
            r
        }
        Err(e) => {
            let msg = format!("{}", e);
            warn!("  ✗ Failed {} ({:?}): {}", relative, elapsed, msg);
            AssetResult {
                relative_path: relative.to_string(),
                success: false,
                output_paths: vec![],
                error: Some(msg),
            }
        }
    }
}

impl Runner {
    pub fn new(db: AssetDb) -> Self {
        Self { db }
    }

    /// Run the full pipeline once
    pub fn run_once(
        &self,
        config: &crate::config::PipelineConfig,
        clean: bool,
    ) -> crate::Result<Vec<AssetResult>> {
        let run_id = Uuid::new_v4().to_string();
        self.db.create_run(&run_id)?;

        let source_root = &config.source.root;
        if !source_root.exists() {
            return Err(RiftError::Pipeline(format!(
                "Source root does not exist: {}",
                source_root.display()
            )));
        }

        info!(
            "Pipeline '{}' starting — scanning {}",
            config.pipeline.name,
            source_root.display()
        );

        if clean {
            info!("  --clean mode: forcing re-conversion of all assets");
        }

        // Pre-run hook
        if let Some(ref cmd) = config.hooks.pre_run {
            run_hook("pre_run", cmd);
        }

        // Phase 1: collect jobs (sequential, fast)
        let jobs = self.collect_jobs(source_root, config, clean)?;

        if jobs.is_empty() {
            info!("  No assets to process (all up to date)");
            let results = vec![];
            self.db.complete_run(&run_id, 0, 0)?;
            // Post-run hook still fires even if nothing to do
            if let Some(ref cmd) = config.hooks.post_run {
                run_hook("post_run", cmd);
            }
            return Ok(results);
        }

        info!("  Found {} assets to process", jobs.len());

        // Phase 2: process assets in parallel with progress bar
        let pb = ProgressBar::new(jobs.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}  {msg}",
                )
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_message("converting...");
        pb.tick();

        let completed = Arc::new(AtomicUsize::new(0));
        let pb_arc = Arc::new(pb);

        let results: Vec<AssetResult> = jobs
            .par_iter()
            .map(|job| {
                let rule = &config.rules[job.rule_index];
                let convert_type = rule.convert.as_deref().unwrap_or("copy");
                pb_arc.set_message(format!("{} — {}", job.relative, convert_type));
                let result = process_asset(&job.path, &job.relative, rule, config);
                let done = completed.fetch_add(1, Ordering::SeqCst) + 1;
                pb_arc.set_position(done as u64);
                result
            })
            .collect();

        pb_arc.finish_and_clear();

        // Phase 3: write results to DB (sequential)
        let converted = results.iter().filter(|r| r.success).count() as u32;
        let errors = results.iter().filter(|r| !r.success).count() as u32;

        for result in &results {
            if result.success {
                if let Err(e) = self.db.mark_converted(&result.relative_path) {
                    error!("DB error marking {} converted: {}", result.relative_path, e);
                }
            } else {
                let err_msg = result.error.as_deref().unwrap_or("unknown error");
                if let Err(e) = self.db.mark_error(&result.relative_path, err_msg) {
                    error!("DB error marking {} error: {}", result.relative_path, e);
                }
            }
        }

        self.db.complete_run(&run_id, converted, errors)?;
        info!(
            "Pipeline complete: {} converted, {} errors",
            converted, errors
        );

        // Post-run or on-error hook
        if errors > 0 {
            if let Some(ref cmd) = config.hooks.on_error {
                run_hook("on_error", cmd);
            }
        } else if let Some(ref cmd) = config.hooks.post_run {
            run_hook("post_run", cmd);
        }

        Ok(results)
    }

    /// Phase 1: walk source dir, match rules, check caches, return jobs
    fn collect_jobs(
        &self,
        source_root: &Path,
        config: &crate::config::PipelineConfig,
        clean: bool,
    ) -> crate::Result<Vec<AssetJob>> {
        let mut jobs = Vec::new();

        for entry in WalkDir::new(source_root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let relative = path
                .strip_prefix(source_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Match rules
            let rule_index = config
                .rules
                .iter()
                .position(|rule| Pattern::new(&rule.pattern).is_ok_and(|p| p.matches(&relative)));

            if let Some(idx) = rule_index {
                let file_modified = modified_iso(path);
                let hash = AssetDb::hash_file(path)?;

                self.db.upsert_asset(&relative, &hash, &file_modified)?;

                if !clean && !self.db.needs_conversion(&relative)? {
                    let rule = &config.rules[idx];
                    let convert_type = rule.convert.as_deref().unwrap_or("copy");
                    info!("  ∎ Skipping (unchanged): {} [{}]", relative, convert_type);
                    continue;
                }

                jobs.push(AssetJob {
                    path: path.to_path_buf(),
                    relative,
                    rule_index: idx,
                });
            }
        }

        Ok(jobs)
    }
}

fn modified_iso(path: &Path) -> String {
    path.metadata()
        .and_then(|m| m.modified())
        .map(|t| {
            let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let secs = duration.as_secs();
            chrono::DateTime::from_timestamp(secs as i64, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "unknown".to_string())
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Run a hook shell command, logging the result
fn run_hook(name: &str, cmd: &str) {
    info!("  ⚡ Hook [{}]: {}", name, cmd);
    let start = Instant::now();
    match std::process::Command::new("sh")
        .args(["-c", cmd])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(output) => {
            let elapsed = start.elapsed();
            if output.status.success() {
                info!("  ✓ Hook [{}] completed ({:?})", name, elapsed);
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.trim().is_empty() {
                    info!("  Hook output:\n{}", stdout);
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!(
                    "  ✗ Hook [{}] exited with code {} ({:?}): {}",
                    name,
                    output.status.code().unwrap_or(-1),
                    elapsed,
                    stderr.trim()
                );
            }
        }
        Err(e) => {
            error!("  ✗ Hook [{}] failed to execute: {}", name, e);
        }
    }
}

/// File watcher that triggers pipeline on changes
pub struct Watcher {
    runner: Runner,
}

impl Watcher {
    pub fn new(runner: Runner) -> Self {
        Self { runner }
    }

    pub fn watch(&self, config: &crate::config::PipelineConfig) -> crate::Result<()> {
        use notify::{
            Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher as _,
        };

        let source_root = config.source.root.clone();
        if !source_root.exists() {
            return Err(RiftError::Pipeline(format!(
                "Source root does not exist: {}",
                source_root.display()
            )));
        }

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())?;
        watcher.watch(&source_root, RecursiveMode::Recursive)?;

        info!("👀 Watching {} for changes...", source_root.display());

        // Initial run
        if let Err(e) = self.runner.run_once(config, false) {
            error!("Initial pipeline run failed: {}", e);
        }

        let debounce = Duration::from_millis(500);
        let mut last_trigger = Instant::now();

        for event in rx {
            match event {
                Ok(_event) => {
                    if last_trigger.elapsed() >= debounce {
                        info!("📦 Change detected — running pipeline...");
                        if let Err(e) = self.runner.run_once(config, false) {
                            error!("Pipeline error: {}", e);
                        }
                        last_trigger = Instant::now();
                    }
                }
                Err(e) => {
                    warn!("Watch error: {}", e);
                }
            }
        }

        Ok(())
    }
}
