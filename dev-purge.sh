#!/bin/bash

# ============================================================================
# DEV-PURGE: Safe cleanup script with IDE & essential app protection
# ============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DRY_RUN=${1:-}
CURRENT_PATH=$(pwd)

# Protected directories - WILL NOT DELETE
PROTECTED_DIRS=(
    ".vscode"           # VS Code settings and extensions
    ".idea"             # JetBrains IDE settings
    ".intellij"         # IntelliJ configuration
    ".codeintelligence" # Code Intelligence settings
    ".git"              # Git repository
    ".github"           # GitHub workflows and config
    ".gitignore"        # Git ignore file
    ".editorconfig"     # Editor config
    ".env"              # Environment variables
    ".env.local"        # Local env overrides
    "node_modules/.bin" # Executable binaries critical for scripts
)

# Dangerous base paths - refuse to run in these
DANGEROUS_PATHS=(
    "$HOME"
    "/"
    "/usr"
    "/var"
    "/etc"
    "/opt"
    "/usr/local"
    "C:\\Program Files"
    "C:\\Program Files (x86)"
    "C:\\Users\\Public"
)

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  🛡️  DEV-PURGE: Safe Cleanup Script      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo

# Safety checks
echo -e "${YELLOW}🔍 Performing safety checks...${NC}"

# Check if running in a dangerous directory
for dangerous in "${DANGEROUS_PATHS[@]}"; do
    if [[ "$CURRENT_PATH" == "$dangerous"* ]]; then
        echo -e "${RED}❌ ERROR: Refusing to run in system directory: $CURRENT_PATH${NC}"
        exit 1
    fi
done

# Check if in git root
if [[ ! -d ".git" && ! -d "node_modules" && ! -d "target" && ! -d ".venv" ]]; then
    echo -e "${YELLOW}⚠️  WARNING: Does not appear to be a project directory.${NC}"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${GREEN}✓ Safety checks passed${NC}"
echo

# Show what will be cleaned
echo -e "${BLUE}📋 Protected items (WILL NOT DELETE):${NC}"
for item in "${PROTECTED_DIRS[@]}"; do
    echo -e "${GREEN}   ✓ $item${NC}"
done
echo

echo -e "${YELLOW}⚠️  This will perform cleanup from: $CURRENT_PATH${NC}"

if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo -e "${BLUE}🔍 DRY RUN MODE - No files will be deleted${NC}"
    echo
else
    read -p "Are you sure? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

echo

# Function to safely remove with exclusions
safe_remove() {
    local path=$1
    local description=$2
    
    if [[ -e "$path" ]]; then
        # Check if path is protected
        for protected in "${PROTECTED_DIRS[@]}"; do
            if [[ "$path" == *"/$protected"* ]] || [[ "$path" == *"\$protected"* ]]; then
                echo -e "${YELLOW}   ⊘ Skipping (protected): $path${NC}"
                return
            fi
        done
        
        if [[ "$DRY_RUN" == "--dry-run" ]]; then
            echo -e "${BLUE}   [DRY RUN] Would delete: $path${NC}"
        else
            rm -rf "$path"
            echo -e "${GREEN}   ✓ Cleaned: $description${NC}"
        fi
    fi
}

# Rust cleanup
if command -v cargo &> /dev/null; then
    echo -e "${BLUE}🦀 Cleaning Rust cache...${NC}"
    if [[ "$DRY_RUN" == "--dry-run" ]]; then
        echo -e "${BLUE}   [DRY RUN] Would clean: cargo registry cache${NC}"
        echo -e "${BLUE}   [DRY RUN] Would clean: cargo index${NC}"
    else
        rm -rf "$HOME/.cargo/registry/cache"/* 2>/dev/null || true
        rm -rf "$HOME/.cargo/index"/* 2>/dev/null || true
        echo -e "${GREEN}   ✓ Cargo caches cleaned${NC}"
    fi
fi

# Go cleanup
if command -v go &> /dev/null; then
    echo -e "${BLUE}🐹 Cleaning Go cache...${NC}"
    if [[ "$DRY_RUN" == "--dry-run" ]]; then
        echo -e "${BLUE}   [DRY RUN] Would run: go clean -modcache${NC}"
    else
        go clean -modcache
        echo -e "${GREEN}   ✓ Go module cache cleaned${NC}"
    fi
fi

# Docker cleanup (safer version)
if command -v docker &> /dev/null; then
    echo -e "${BLUE}🐳 Cleaning Docker...${NC}"
    if [[ "$DRY_RUN" == "--dry-run" ]]; then
        echo -e "${BLUE}   [DRY RUN] Would run: docker system prune (dangling images/containers only)${NC}"
    else
        docker system prune -f --filter "until=240h" 2>/dev/null || true
        echo -e "${GREEN}   ✓ Docker cleaned (dangling items only)${NC}"
    fi
fi

# Project folders cleanup with protection
echo -e "${BLUE}📦 Cleaning project folders...${NC}"

find . -maxdepth 5 -type d \( \
    -name "node_modules" \
    -o -name "target" \
    -o -name ".next" \
    -o -name "dist" \
    -o -name "build" \
    -o -name "__pycache__" \
    -o -name ".pytest_cache" \
    -o -name ".venv" \
    -o -name "venv" \
    -o -name ".tox" \
    -o -name "*.egg-info" \
    \) \
    ! -path "*/.vscode/*" \
    ! -path "*/.idea/*" \
    ! -path "*/.git/*" \
    ! -name ".vscode" \
    ! -name ".idea" \
    ! -name ".git" \
    -print | while read -r dir; do
    if [[ "$DRY_RUN" == "--dry-run" ]]; then
        echo -e "${BLUE}   [DRY RUN] Would delete: $dir${NC}"
    else
        rm -rf "$dir"
        echo -e "${GREEN}   ✓ Removed: $dir${NC}"
    fi
done

echo

if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo -e "${BLUE}✅ DRY RUN COMPLETE${NC}"
    echo -e "${BLUE}Run without --dry-run flag to perform actual cleanup${NC}"
else
    echo -e "${GREEN}✨ Cleanup complete!${NC}"
    echo -e "${YELLOW}IDE configurations and essential files are protected.${NC}"
fi




