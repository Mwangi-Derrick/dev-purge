# Contributing to DEV-PURGE

Thanks for considering contributing to **DEV-PURGE**! We're building the modern solution to developer disk bloat, and your help makes it better.

## 🎯 How to Contribute

### 1. Report Issues
Found a bug or have a feature request?

- **Check existing issues** first (someone might've already reported it)
- Open a new [GitHub Issue](https://github.com/Mwangi-Derrick/dev-purge/issues) with:
  - Clear title (e.g., "Script crashes on macOS with non-ASCII filenames")
  - Steps to reproduce
  - Your OS and shell (Git Bash, Zsh, Bash, etc.)
  - The output/error message

### 2. Suggest Features
Have an idea for a killer feature? Open a [GitHub Discussion](https://github.com/Mwangi-Derrick/dev-purge/discussions) or Issue tagged `enhancement`.

**Ideas we're tracking:**
- Rust rewrite for parallel scanning
- Configuration file support (`.purgeignore`)
- Size reporting (show exactly how much each folder saved)
- Watch mode (auto-clean when disk < 10% free)
- Docker-specific cleanup modes
- Integration with CI/CD pipelines

### 3. Submit a Pull Request

#### Step 1: Fork & Clone
```bash
git clone https://github.com/yourusername/dev-purge.git
cd dev-purge
```

#### Step 2: Create a Branch
```bash
git checkout -b feature/your-awesome-feature
```

Branch naming:
- `feature/` for new features
- `fix/` for bug fixes
- `docs/` for documentation updates

#### Step 3: Make Changes

**For script changes:**
- Test in `--dry-run` mode first
- Ensure it still protects IDE directories
- Add a comment if the logic isn't obvious

**For documentation:**
- Keep it concise
- Use examples where helpful
- Link to related sections

#### Step 4: Test Thoroughly

```bash
# Test the dry-run
./dev-purge.sh --dry-run

# Test in an isolated directory with sample junk
mkdir ~/test-purge-dir
cd ~/test-purge-dir
mkdir -p target node_modules
echo "test" > target/junk.txt
~/dev-purge.sh --dry-run
~/dev-purge.sh
```

#### Step 5: Commit & Push

```bash
git add .
git commit -m "feat: add support for X"  # or fix: bug fix, or docs: update
git push origin feature/your-awesome-feature
```

#### Step 6: Open a Pull Request

On GitHub:
1. Go to your fork
2. Click "Compare & pull request"
3. Write a clear description of your changes
4. Reference any related issues (#123)

---

## 📋 Contribution Checklist

Before submitting:

- [ ] Changes have been tested (`--dry-run` works?)
- [ ] IDE directories are still protected
- [ ] No new errors or permission issues
- [ ] Documentation is updated if applicable
- [ ] Commit messages are clear
- [ ] No breaking changes (or clearly noted in PR)

---

## 🚀 Roadmap Contributions

### High Priority (Help Wanted!)

1. **Rust Rewrite** (`purge-rs`)
   - We want to make this **10x faster** with parallel scanning
   - Using `walkdir` + `rayon` for multi-threaded traversal
   - Status: Early design stage
   - Interested? Open an issue to discuss architecture

2. **Cross-Platform Testing**
   - Test on Windows (Git Bash, PowerShell, WSL)
   - Test on macOS (Intel & Apple Silicon)
   - Test on Linux (Ubuntu, Fedora, Arch)

3. **Docker Improvements**
   - Add safer Docker pruning modes
   - Support for `docker-compose` cleanup
   - Preserve volumes intelligently

### Medium Priority

4. **Configuration File** (`.purgeignore`)
   - Let users define custom exclusions
   - Similar to `.gitignore` format

5. **Metrics & Reporting**
   - Show before/after disk usage
   - Export cleanup summary as JSON/CSV
   - Track total space saved over time

---

## 💬 Code Style

Keep it simple and readable:

```bash
# ✅ Good
if command -v cargo &> /dev/null; then
    echo "🦀 Cleaning Rust..."
    rm -rf "$HOME/.cargo/registry/cache"/*
fi

# ❌ Avoid
if [ -x "$(command -v cargo)" ]; then rm -rf $HOME/.cargo/registry/cache/*; fi
```

**Guidelines:**
- Add comments for non-obvious logic
- Use descriptive variable names
- Keep emoji for visual clarity but not overuse
- Test edge cases (paths with spaces, permissions, etc.)

---

## 🧪 Testing Your Changes

### Safety First
Always test destructive operations in isolation:

```bash
# Create a test project
mkdir ~/test-dev-purge
cd ~/test-dev-purge
mkdir -p src/target src/node_modules
touch src/target/build.log
touch src/node_modules/package.json

# Run dry-run
bash ~/dev-purge.sh --dry-run

# Verify it found the targets
# Then run actual cleanup if confident
```

### Cross-Platform
If possible, test on:
- Windows (Git Bash)
- macOS (Zsh/Bash)
- Linux (Ubuntu/Fedora)

---

## ❓ Questions?

- **GitHub Discussions:** Ask anything technical or design-related
- **GitHub Issues:** Report bugs or request features
- **Pull Request Comments:** Discuss changes directly

---

## 📝 Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add support for Poetry cache cleanup
fix: handle paths with spaces correctly
docs: update README with Docker tips
test: add edge case tests for permission errors
```

---

## 🎉 Contributors

Thanks to everyone who's contributed! You're helping developers reclaim their SSD space one cleanup at a time.

---

## License

By contributing, you agree your work is licensed under the MIT License (same as the project).

Happy coding! 🚀
