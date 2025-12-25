# Rust Config Guardian

**A lightweight, fast CLI tool to detect configuration drift**

Rust Config Guardian helps you monitor critical configuration directories (e.g., `/etc/nginx`, `/etc/apache2`, system configs, app settings) by taking cryptographic snapshots of files and alerting you to any unintended changes.

Perfect for DevOps engineers, sysadmins, and security-conscious teams who want to ensure infrastructure remains in a known good state.

## ✨ Features

- **Snapshot** – Create a baseline of file contents using SHA-256 hashes
- **Compare** – Detect new, modified, or deleted configuration files
- **Monitor** – Real-time file system watching with automatic drift checks
- **Robust error handling** – Skips unreadable files with clear warnings
- **Logging** – All operations and drift events logged to `drift.log`
- **Clean CLI** – Built with `clap` for intuitive usage
- **Zero external runtime dependencies** – Pure Rust, single binary

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/NPC-AYUSH/Rust-Config-Guardian.git
cd Rust-Config-Guardian
cargo build --release
sudo cp target/release/config_drift_detector /usr/local/bin/config-guardian
```

### Usage

```bash
# 1. Take an initial snapshot of a config directory
config-guardian snapshot /etc/nginx

# 2. Check for drift later
config-guardian compare /etc/nginx

# Output example:
# Drift detected:
#   Changed: /etc/nginx/nginx.conf
#   New: /etc/nginx/sites-enabled/my-new-site.conf
#   Deleted: /etc/nginx/sites-enabled/old-site.conf

# 3. Run continuous monitoring (great for live detection)
config-guardian monitor /etc/nginx
# → Watches for changes and reports drift in real time (Ctrl+C to stop)

```
