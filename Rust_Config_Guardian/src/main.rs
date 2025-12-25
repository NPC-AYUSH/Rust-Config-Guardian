use clap::Parser;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use sha2::{Digest, Sha256};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

mod utils;
use utils::is_valid_directory;

#[derive(Serialize, Deserialize)]
struct FileHash {
    path: String,
    hash: String,
}

#[derive(Parser)]
#[command(author, version, about = "Detect configuration drift in files.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Take a snapshot of configuration files.
    Snapshot {
        #[arg(value_name = "DIRECTORY")]
        directory: Option<String>,
    },
    /// Compare current files with the last snapshot.
    Compare {
        #[arg(value_name = "DIRECTORY")]
        directory: Option<String>,
        #[arg(long, action)]
        alert: bool,
    },
    /// Monitor directory for changes and detect drift.
    Monitor {
        #[arg(value_name = "DIRECTORY")]
        directory: Option<String>,
        #[arg(long, action)]
        alert: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("drift.log")?,
    )?;
    log::info!("Configuration Drift Detector started.");

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Snapshot { directory }) => {
            let dir = directory.as_deref().unwrap_or(".");
            if !is_valid_directory(dir) {
                return Err("Provided path is not a valid directory.".into());
            }
            log::info!("Taking snapshot of directory: {}", dir);
            let snapshot = take_snapshot(dir)?;
            println!(
                "Snapshot taken and saved to snapshot.json ({} files)",
                snapshot.len()
            );
        }
        Some(Commands::Compare { directory, alert }) => {
            let dir = directory.as_deref().unwrap_or(".");
            if !is_valid_directory(dir) {
                return Err("Provided path is not a valid directory.".into());
            }
            log::info!("Comparing directory: {} (alert: {})", dir, alert);
            compare_with_snapshot(dir, *alert)?;
        }
        Some(Commands::Monitor { directory, alert }) => {
            let dir = directory.as_deref().unwrap_or(".");
            if !is_valid_directory(dir) {
                return Err("Provided path is not a valid directory.".into());
            }
            log::info!("Monitoring directory: {} (alert: {})", dir, alert);
            monitor_directory(dir, *alert)?;
        }
        None => {
            println!("No command provided. Use --help for options.");
        }
    }

    Ok(())
}

fn take_snapshot(dir: &str) -> Result<Vec<FileHash>, Box<dyn std::error::Error>> {
    let mut hashes = Vec::new();

    let dir_entries = fs::read_dir(dir);
    if let Ok(entries) = dir_entries {
        if entries.count() == 0 {
            println!("Warning: Directory {} is empty.", dir);
        }
    }

    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: Could not read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if path.is_file() {
            match fs::read(&path) {
                Ok(content) => {
                    let mut hasher = Sha256::new();
                    hasher.update(&content);
                    let hash = format!("{:x}", hasher.finalize());
                    hashes.push(FileHash {
                        path: path.to_string_lossy().into_owned(),
                        hash,
                    });
                }
                Err(e) => eprintln!("Warning: Could not read file {}: {}", path.display(), e),
            }
        }
    }

    let json = serde_json::to_string_pretty(&hashes)?;
    fs::write("snapshot.json", json)?;
    Ok(hashes)
}

fn compare_with_snapshot(dir: &str, _alert: bool) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_data = fs::read_to_string("snapshot.json");
    let snapshot: Vec<FileHash> = match snapshot_data {
        Ok(data) => serde_json::from_str(&data)?,
        Err(_) => return Err("No snapshot.json found. Run 'snapshot' command first.".into()),
    };

    let current = take_snapshot(dir)?; // Re-uses take_snapshot to get current hashes

    let mut drifts = Vec::new();

    // Detect new or changed files
    for curr in &current {
        if let Some(prev) = snapshot.iter().find(|p| p.path == curr.path) {
            if prev.hash != curr.hash {
                drifts.push(format!("Changed: {}", curr.path));
            }
        } else {
            drifts.push(format!("New: {}", curr.path));
        }
    }

    // Detect deleted files
    for prev in &snapshot {
        if !current.iter().any(|c| c.path == prev.path) {
            drifts.push(format!("Deleted: {}", prev.path));
        }
    }

    if drifts.is_empty() {
        println!("No drift detected.");
        log::info!("No configuration drift detected.");
    } else {
        println!("Drift detected:");
        for drift in &drifts {
            println!("  {}", drift);
        }
        log::warn!("Configuration drift detected: {:?}", drifts);

        // Alert functionality is disabled as requested
        // If you want to re-enable email alerts later, uncomment and configure send_email_alert()
    }

    Ok(())
}

fn monitor_directory(dir: &str, _alert: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, std::time::Duration::from_secs(1))?;
    watcher.watch(Path::new(dir), RecursiveMode::NonRecursive)?;

    println!("Monitoring {} for changes... (Press Ctrl+C to stop)", dir);

    let mut last_check = Instant::now();
    const DEBOUNCE_INTERVAL: Duration = Duration::from_secs(2);

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if last_check.elapsed() >= DEBOUNCE_INTERVAL {
                    println!("Change detected: {:?}", event);
                    let _ = compare_with_snapshot(dir, _alert);
                    last_check = Instant::now();
                }
            }
            Ok(Err(e)) => println!("Watch error: {:?}", e),
            Err(e) => println!("Channel error: {:?}", e),
        }
    }
}
