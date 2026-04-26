# Contributing to DEV-PURGE 🔥

Yo. You found the project. You liked it enough to contribute. Respect.

`dev-purge` is a **safety-first, high-speed** developer workspace cleanup tool—dual-engine, zero-compromise. We delete build artifacts for a living, and we're really good at it.

Pull up a chair. Let's make this thing better together.

---

## 🏗️ How This Thing Is Built

Two engines. One mission.

| Engine | Location | Why It Exists |
|--------|----------|---------------|
| 🦀 **Rust** | `/rust` | Parallel scanning, rich CLI, production-grade |
| ⚡ **Zig** | `/zig` | Tiny binary, zero deps, blazing distribution |

Both share the same **domain logic and safety heuristics**. If you touch one, you touch both. Symmetry is non-negotiable.

---

## 🎯 What You Can Contribute

### ➕ Add a New Language or Tool

Know a build folder that should be nuked? Add it to the **Compact Registry**.

- 🦀 Rust: `rust/src/domain/patterns.rs` → `ARTIFACT_REGISTRY`
- ⚡ Zig: `zig/src/domain/config.zig` → `ARTIFACT_REGISTRY`

Three pattern types available:

| Type | What It Does | Example |
|------|-------------|---------|
| `Exact` | Matches a dir name exactly | `target` |
| `Prefix` | Matches names that start with a string | `cmake-build-` → matches `cmake-build-debug`, `cmake-build-release` |
| `Guarded` | Only matches if a sibling file/extension exists | `bin/` only if `.csproj` exists → won't kill random `bin` folders |

> 💡 **Don't guess.** Verify the pattern against real projects before submitting.
> A match that's too broad is worse than no match at all.

---

### 🛡️ Add OS Protection Rules

Some directories are sacred. Untouchable. Forever.

- 🦀 Rust: `rust/src/domain/os.rs`
- ⚡ Zig: `zig/src/domain/os.zig`

Think `/System`, `C:\Windows`, `~/.ssh`. If deleting it would ruin someone's day (or career), it goes here.

---

## 🛠️ Dev Workflow

We use a `justfile`. Install [`just`](https://github.com/casey/just)—it's small, fast, and worth it.

### 🦀 Rust

```bash
just build-rust     # compile release binary
just test-rust      # run all tests
just lint-rust      # clippy + fmt checks
just fmt-rust       # auto-format
```

### ⚡ Zig

```bash
just build-zig      # optimized native build
just test-zig       # run Zig tests
```

No `just`? Open the `justfile` and run the raw commands. They're not magic.

---

## 🔁 Submitting a PR

```bash
# 1. Fork the repo and clone
git clone https://github.com/<your-handle>/dev-purge.git
cd dev-purge

# 2. Branch off main
git checkout -b feature/python-venv-cleanup   # or fix/ or docs/

# 3. Make your changes (both engines if applicable)

# 4. Dry-run to verify
./dev-purge --dry-run

# 5. Run tests and linting
just test-rust
just lint-rust

# 6. Commit using conventional commits
git commit -m "feat: add support for Python venv artifacts"

# 7. Push and open PR
git push origin feature/python-venv-cleanup
```

**Commit message format:**

```
feat: add X
fix: handle Y edge case
docs: clarify Z
test: cover W scenario
```

---

## ✅ PR Checklist

Before you open that PR, run through this:

- [ ] Changes implemented in **both** Rust and Zig engines (no engine left behind)
- [ ] `--dry-run` shows exactly what you expect—nothing more, nothing less
- [ ] `just test-rust` passes clean
- [ ] `just lint-rust` is happy (no clippy warnings)
- [ ] PR description explains *why* this pattern or rule is needed, not just what it does
- [ ] No overly broad patterns that could match unintended directories

---

## 🧪 Safety First. Always.

This tool **deletes things**. That's the whole point.

But deleting the wrong things is a support ticket, a lost project, or a very bad afternoon. So when you add a new rule, sanity check it hard:

1. **Too broad?** Does `tmp` match things you didn't expect? Test it.
2. **False positives?** Does it incorrectly match production-like project structures?
3. **OS protections respected?** Your rule shouldn't fire inside protected system paths.

**The golden rule:** `--dry-run` first. Then `--dry-run` again. *Then* run for real.

---

## 🐛 Reporting Bugs

Open a [GitHub Issue](https://github.com/Mwangi-Derrick/dev-purge/issues) with:

- A clear title (e.g., `"Crashes on paths with spaces on macOS"`)
- Steps to reproduce
- Your OS and shell (Git Bash, Zsh, PowerShell, WSL, etc.)
- The full error output

---

## 💬 Questions? Ideas? Just want to talk shop?

Open a [GitHub Discussion](https://github.com/Mwangi-Derrick/dev-purge/discussions) or file an issue tagged `help wanted`.

We respond faster than `cargo clean`.

---

*Built for developers, by developers. Go save some disk space.* 🚀