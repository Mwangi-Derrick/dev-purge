# 🛡️ DEV-PURGE: Smart Disk Space Recovery for Developers

> **"I was down to 0MB of disk space while building my startup. I ran this tool and recovered 30GB in seconds. No more 'Disk Full' panics during builds."**

---

## 🚀 The Problem

If you are a multi-stack developer juggling (**Rust 🦀**· **Go 🐹**  · **Node.js  📦** . **Python 🐍** · **Docker 🐳**), **Java **, **Swift**, **.NET**, **Elixir**, and more—your SSD is silently **hemorrhaging space**.

### Where the 30GB is hiding:

| Component | Space Eaten | Why It Happens |
|-----------|-------------|---|
| **Rust `target/` folders** | 15–20 GB | Every `cargo build` creates a fresh build artifact directory per project |
| **Node `node_modules`** | 5–8 GB | Heavy dependencies (especially Next.js/React stacks) multiply across projects |
| **Go module cache** | 2–3 GB | Global `$GOMODCACHE` stores every version of every dependency ever pulled |
| **Python `__pycache__` and `.venv/`** | 2–4 GB | Virtual environments and bytecode accumulate silently across 10+ projects |
| **Java/Gradle/Maven build dirs** | 3–5 GB | `target/`, `build/`, `.gradle` cache never cleaned up |
| **.NET `bin/obj` folders** | 2–3 GB | Every `dotnet build` leaves artifacts behind |
| **Docker layers** | 2–5 GB | Old images, dangling layers, container logs that never rotated |

**The killer?** None of this is obvious. Your IDE keeps running. Your builds work. Then one morning: **0MB free**. 💀

---

## ✨ The Solution

**DEV-PURGE** is a **modern, fortified and engineered cleanup tool** available in two flavors:

### Rust Version (Feature-Rich)
✅ **Parallel scanning** with Rayon for blazing speed  
✅ **Trait-based architecture** for extensibility (easy contributor model)  
✅ **Comprehensive safety** with OS-aware path protection  
✅ **30+ build tool patterns** across 15+ programming languages  
✅ **Color-coded output** with real-time progress  
✅ **Dry-run mode** to preview before deleting  

### Zig Version (Lightweight)
⚡ **Tiny binary** (~5MB vs 10MB+ Rust)  
⚡ **Fast cross-compilation** for all platforms  
⚡ **Feature parity** with Rust version  
⚡ **No runtime overhead** (same safety & patterns)  

---

## 📥 Installation

### Rust Version (Recommended for Development)

```bash
# Clone and build
git clone https://github.com/Mwangi-Derrick/dev-purge.git
cd dev-purge/rust

# Install globally
cargo install --path .

# Or run directly
cargo run --release -- --help
```

### Zig Version (Lightweight Alternative)

```bash
cd dev-purge/zig

# Build
zig build -Doptimize=ReleaseFast

# Or run directly
zig run src/main.zig -- --help
```

### Pre-Built Binaries

Download from [GitHub Releases](https://github.com/Mwangi-Derrick/dev-purge/releases):
- `dev-purge-linux-x64` - Linux x86_64
- `dev-purge-windows-x64.exe` - Windows
- `dev-purge-macos-x64` - macOS Intel
- `dev-purge-linux-arm64` - Linux ARM64

---

## 🎯 Quick Start

### Preview What Will Be Deleted

```bash
cd ~/your/projects
dev-purge . --dry-run
```

**Output:**
```
🛡️  DEV-PURGE: Safe Cleanup Script
🔍 Performing safety checks...
✓ Safety checks passed

📋 Protected items (WILL NOT DELETE):
   ✓ .vscode
   ✓ .idea
   ✓ .git
   ... (and more)

[DRY RUN] Would delete: ./backend/target
[DRY RUN] Would delete: ./frontend/node_modules
[DRY RUN] Would clean: cargo registry cache

✅ DRY RUN COMPLETE
Run without --dry-run flag to perform actual cleanup
```

### Actually Clean Up

```bash
dev-purge
```

When prompted, type `y` and watch your storage come back. 🎉

---

## 🛡️ What Gets Protected?

This script **will never touch:**

```
.vscode/              # VS Code settings & extensions
.idea/                # JetBrains IDE configuration
.cursor/              # Cursor IDE settings
.git/                 # Your repository (critical!)
.github/              # GitHub workflows
.gitignore            # Git configuration
.editorconfig         # Editor standards
.env / .env.local     # Environment secrets
```

And it **refuses to run** in system directories:
- `/` (root)
- `/usr` / `/var` / `/etc` (Linux system dirs)
- `C:\Program Files` (Windows)

This prevents accidental "oopsies" that could brick your OS.

---

## 🧹 What Gets Cleaned?

### Project Artifacts
- `target/` (Rust builds)
- `node_modules/` (Node.js)
- `.next/` (Next.js builds)
- `dist/` / `build/` (General build outputs)
- `__pycache__/` (Python bytecode)
- `.pytest_cache/` (Test artifacts)
- `.venv/` / `venv/` (Python virtual environments)

### System Caches
- **Rust:** `.cargo/registry/cache` & `.cargo/index`
- **Go:** Module cache (`go clean -modcache`)
- **Docker:** Dangling images & containers (safely—preserves running services)

---

## ⚙️ Usage Options

```bash
# Preview what will be deleted
dev-purge --dry-run

# Actually clean up
dev-purge

# Show help
dev-purge --help
```

---

## 🌐 Rust + Zig: Two versions, one safe cleanup experience

The repo contains both a Rust flagship and a compact Zig CLI:

- `rust/` is the feature-rich version with robust scanning, safety checks, and user-friendly output.
- `zig/` is designed for tiny downloads and fast cross-platform releases.

Why Zig matters for dev-purge:

- Zig binaries are much smaller than Rust's static std binaries.
- Zig cross-compilation is simple and works without extra toolchains.
- Zig is ideal for a lightweight installer on slow connections.

If you want the best of both worlds, the Rust version is the rich CLI and the Zig version is the fast-download release.

---

## 🏗️ Rust Version: Trait-Based Architecture

The Rust implementation uses a **modular, extensible design** built around traits for easy contribution:

### Core Traits

- **`Scanner`**: Defines how to scan directories for artifacts
- **`SafetyChecker`**: Validates paths are safe to delete
- **`Cleaner`**: Handles the actual deletion logic

### Adding New Scanners

To add support for a new tool (e.g., Docker artifacts):

```rust
use dev_purge::domain::traits::{Scanner, ScanResult};

pub struct DockerScanner;

impl Scanner for DockerScanner {
    fn scan(&self, root: &std::path::Path) -> anyhow::Result<Vec<ScanResult>> {
        // Scan for Docker-related artifacts
        Ok(vec![])
    }
}
```

### Architecture Benefits

- **Extensible**: Add new scanners without touching core logic
- **Testable**: Each trait can be unit tested independently
- **Safe**: Safety checks are enforced by the trait system
- **Parallel**: Built-in parallel scanning with Rayon

The trait-based design makes it easy for contributors to extend dev-purge for new build tools and frameworks.

---

## 🚨 Real-World Example: The 30GB Win

```bash
$ cd ~/projects
$ dev-purge --dry-run

🛡️  DEV-PURGE: Safe Cleanup Script
📋 Protected items (WILL NOT DELETE):
   ✓ .vscode
   ✓ .git
   ... (12 more items)


[DRY RUN] Would delete: .*/target
[DRY RUN] Would delete: .*/node_modules
[DRY RUN] Would clean: cargo registry cache (estimated 8GB)

# Estimated space to recover: ~30GB

$ dev-purge
# After cleanup:
✨ Cleanup complete!
IDE configurations and essential files are protected.

# Check your space
$ df -h
Filesystem      Size  Used Avail Use%
/dev/nvme0n1   476G   180G  296G  38%  # Was 100% full!
```

---

## 🤖 Automation: Run on Startup

### Windows (Task Scheduler)

1. Press `Win + R`, type `taskschd.msc`
2. Create Basic Task → Name: "Dev Purge"
3. Trigger: On startup (or weekly at 6 PM)
4. Action: Start program
   - Program: `C:\Program Files\Git\bin\bash.exe`
   - Arguments: `-c "~/dev-purge.sh --dry-run"`

### Linux / macOS (Cron)

```bash
# Run every Sunday at midnight
crontab -e
```

Add this line:
```cron
0 0 * * 0 /usr/local/bin/dev-purge >> /var/log/dev-purge.log 2>&1
```

### Automatic on Terminal Startup

Add to `~/.bashrc`:
```bash
# Optional: Run silently on terminal start (run in background)
# ( dev-purge --dry-run > /dev/null 2>&1 & )
```

---

## 🐳 Docker-Specific Note

If you're using Docker for databases (MongoDB, Minio, EMQX), this tool handles it **safely**:

- Removes only **dangling images** & **old layers** (not running containers)
- Respects **volumes** with active data
- Logs Docker cleanup attempt in output

⚠️ **Critical:** Docker log bloat is the #1 SSD killer on dev machines. If space disappears overnight, check:

```bash
du -hs /var/lib/docker/
docker system df
```

Configure log limits in `/etc/docker/daemon.json`:
```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  }
}
```

Then restart: `sudo systemctl restart docker`

---

## 🔮 Roadmap: The Path to v1.0 (The Surgical Update)

Dev-Purge is moving towards a professional, server-ready `v1.0` release. Our strategy focuses on **Zero-Blame Safety** and **Global Distribution**.

### 🛠️ v1.0 Objectives

- [x] **Surgical Safety (The "Trash" Rule)**: 
  - Integrate with the OS trash bin instead of permanent deletion. If an "oopsie" happens, just hit Restore.
  - Implement a `--force` flag requirement for non-interactive deletions.
- [x] **Server-Grade Cleaning (The "Deep" Update)**:
  - [x] **Docker API Integration**: Use `bollard` (Rust) to prune logs and volumes without touching the filesystem directly.
  - [ ] **Large Log Rotation**: Auto-truncate massive `.log` files in common server directories.
- [ ] **Professional Distribution**:
  - [ ] Publish to **Crates.io** (The Rust Flagship).
  - [ ] Add **Homebrew** and **Apt** repository support.
  - [ ] Generate shell completions (Zsh/Fish/Bash).
- [ ] **The Interactive Experience (TUI)**:
  - [ ] Integrate **Ratatui** for a dashboard-style UI.
  - [ ] **Selective Purging**: Allow users to toggle specific folders via a list view.
  - [ ] **Real-time Metrics**: Visual breakdown of disk usage by language/tool.
- [ ] **Documentation**:
  - [ ] Write the **"Safety Manifesto"**: A deep dive into our 40+ protection rules.
  - [ ] "Dev-Purge for Servers" deployment guide.
x
---

## 📋 Safety Features

| Feature | Purpose |
|---------|---------|
| **Dry-Run Mode** | Preview deletions without touching files |
| **Permission Checks** | Refuses to run in dangerous directories |
| **Protected List** | Hardcoded safelist for critical IDE/config dirs |
| **User Confirmation** | Requires explicit `y` confirmation before cleanup |
| **Colorized Output** | Clear visual feedback on what's happening |
| **Error Silencing** | Permission denied errors don't clutter output |

---

## 🐛 Troubleshooting

### "Permission Denied" Errors in Output

Normal on Windows/Git Bash—the script safely ignores system folder permission errors. They won't prevent cleanup.

### VS Code / IDE Stops Working After Cleanup

This can happen if the IDE was storing cache in a deleted folder. **Quick fix:**

```bash
# Reinstall your language servers
cargo check         # Rust
go mod tidy         # Go
npm install         # Node.js

# Restart the IDE
```

If problems persist, reinstall the IDE (your code is safe—only caches were deleted).

### Script Isn't Finding Anything to Delete

You might already be clean! Or, run it from a different directory:

```bash
cd ~/your/projects  # Not from home (~)
dev-purge --dry-run
```

Running from your projects directory is 10x faster than scanning your entire home folder.

---

## 💡 Tips for Maximum Productivity

1. **Run weekly** during downtime (not during active builds)
2. **Use `--dry-run` first** to understand what will be deleted
3. **Monitor Docker logs** separately (they rot faster than code)
4. **Set up cron** so you never think about it again
5. **Export the cleaned space** mentally as "freed compute"—your SSD will thank you with faster writes

---

## 🤝 Contributing

Got ideas? Found a bug? Want to help build the Rust version?

1. Fork this repo
2. Create a feature branch (`git checkout -b feature/amazing-thing`)
3. Make your changes
4. Test with `./dev-purge.sh --dry-run` in a test directory
5. Submit a pull request

**Ideas for contributions:**
- Add support for other package managers (Poetry, Cargo Binstall, etc.)
- Build the Rust CLI (`cargo new devpurge`)
- Write tests
- Expand documentation for specific IDEs

---

## 📊 Stats

- **Space Recovered (per user):** 20–50 GB
- **Time to Run:** 5–15 seconds
- **Risk Level:** Very Low (with `--dry-run` mode)
- **IDE Protection:** 100% (verified with VS Code, JetBrains, Cursor)

---

## 📝 License

MIT License—feel free to use, modify, and redistribute.

---

## 🙏 Acknowledgments

Built for developers juggling **Rust**, **Go**, **Node.js**, and **Docker** across multiple projects. If you're building startups or infrastructure, you know the pain. This script is the answer.

Inspired by real incidents of "0MB SSD panic" during critical builds. 💀

---

## 🚀 Next Steps

1. **Try it:** `dev-purge --dry-run` in your projects folder
2. **Use it:** `dev-purge` when you're ready to reclaim space
3. **Automate it:** Set up cron or Task Scheduler (never worry again)
4. **Share it:** ⭐ Star this repo if it saves you 20+ GB!

---

**Questions?** Open an issue or check the [GitHub Discussions](https://github.com/Mwangi-Derrick/dev-purge/discussions).

**Found a bug?** Submit a [GitHub Issue](https://github.com/Mwangi-Derrick/dev-purge/issues).

Happy coding. Your SSD will thank you. 🎉
