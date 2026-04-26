const std = @import("std");

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2 or std.mem.eql(u8, args[1], "--help") or std.mem.eql(u8, args[1], "-h")) {
        std.debug.print("Usage: {s} <path> [--dry-run]\n", .{args[0]});
        std.debug.print("  <path>: Directory to scan for dev artifacts\n", .{});
        std.debug.print("  --dry-run: Show what would be deleted without actually deleting\n", .{});
        std.debug.print("\nExamples:\n", .{});
        std.debug.print("  {s} . --dry-run    # Preview cleanup in current directory\n", .{args[0]});
        std.debug.print("  {s} ~/projects     # Clean up projects directory\n", .{args[0]});
        return;
    }

    const root_path = args[1];
    const dry_run = args.len > 2 and std.mem.eql(u8, args[2], "--dry-run");

    // Initial safety check
    try checkRootSafety(root_path);

    std.debug.print("🛡️  DEV-PURGE: Scanning {s} for dev artifacts...\n", .{root_path});
    if (dry_run) {
        std.debug.print("🔍 Dry-run mode: No files will be deleted\n", .{});
    }

    // Comprehensive patterns for multiple languages/frameworks
    const patterns = [_][]const u8{
        // Core / Common
        "target",              "node_modules",  "dist",        "build",              "out",

        // Python
        "__pycache__",         ".venv",         "venv",        ".pytest_cache",      ".mypy_cache",
        ".ruff_cache",         ".tox",          ".hypothesis", ".ipynb_checkpoints",

        // JS/TS/Web Ecosystem
        ".next",
        ".nuxt",               ".parcel-cache", ".turbo",      ".nx",                ".svelte-kit",
        ".astro",              ".vite",         ".angular",    ".vercel",

        // Mobile/Other
                   ".dart_tool",
        "zig-cache",           "zig-out",

        // Infrastructure/DevOps
              ".terraform",  ".gradle",

        // Build systems
                   "cmake-build-debug",
        "cmake-build-release",
    };

    var total_size: u64 = 0;
    var count: usize = 0;
    var errors: usize = 0;

    var dir = try std.fs.cwd().openDir(root_path, .{ .iterate = true });
    defer dir.close();

    var walker = try dir.walk(allocator);
    defer walker.deinit();

    while (try walker.next()) |entry| {
        if (entry.kind != .directory) continue;

        const name = std.fs.path.basename(entry.path);

        // Skip protected paths
        if (isProtectedPath(name)) continue;

        for (patterns) |pattern| {
            if (std.mem.eql(u8, name, pattern)) {
                // Estimate size
                const size = estimateDirSize(entry.dir, name, allocator) catch |err| {
                    std.debug.print("⚠️  Error estimating size for {s}: {}\n", .{ entry.path, err });
                    errors += 1;
                    continue;
                };

                total_size += size;
                count += 1;

                if (dry_run) {
                    std.debug.print("[DRY RUN] Would delete: {s} ({d} bytes)\n", .{ entry.path, size });
                } else {
                    std.debug.print("🗑️  Deleting: {s} ({d} bytes)\n", .{ entry.path, size });
                    entry.dir.deleteTree(name) catch |err| {
                        std.debug.print("❌ Failed to delete {s}: {}\n", .{ entry.path, err });
                        errors += 1;
                        continue;
                    };
                }
                break;
            }
        }
    }

    std.debug.print("\n✅ Scan complete!\n", .{});
    std.debug.print("📊 Found: {d} items ({d} bytes recoverable)\n", .{ count, total_size });
    if (errors > 0) {
        std.debug.print("⚠️  Errors: {d}\n", .{errors});
    }
    if (dry_run) {
        std.debug.print("💡 Run without --dry-run to actually delete files\n", .{});
    }
}

/// Check if the root path is safe to scan
fn checkRootSafety(root_path: []const u8) !void {
    // Check for dangerous system paths
    const dangerous_paths = [_][]const u8{
        "/",           "/usr",              "/etc",                    "/var", "/bin", "/sbin", "/lib",
        "C:\\Windows", "C:\\Program Files", "C:\\Program Files (x86)",
    };

    for (dangerous_paths) |dangerous| {
        if (std.mem.eql(u8, root_path, dangerous)) {
            std.debug.print("❌ Refusing to scan system directory: {s}\n", .{dangerous});
            return error.UnsafePath;
        }
    }

    // Check if it's a home directory
    if (std.mem.eql(u8, root_path, "~") or std.mem.endsWith(u8, root_path, "/home/") or std.mem.endsWith(u8, root_path, "\\Users\\")) {
        std.debug.print("❌ Refusing to scan home directory. Run from a specific project directory.\n", .{});
        return error.UnsafePath;
    }
}

/// Check if a directory name is protected and should not be deleted
fn isProtectedPath(name: []const u8) bool {
    const protected = [_][]const u8{
        // IDE configs
        ".vscode",     ".idea",      ".cursor",         ".config",
        // Version control
        ".git",        ".github",    ".gitignore",      ".editorconfig",
        // Secrets
        ".env",        ".env.local", ".env.production",
        // Tool configs
        ".cargo",
        ".npm-global", ".local",     "go",              ".gradle",
        ".m2",
        // Caches (protect to avoid accidental deletion)
                ".cargo",     ".npm",            ".gradle",
        ".m2",
        // OS-specific
                "Library",    "AppData",         ".cache",
    };

    for (protected) |p| {
        if (std.mem.eql(u8, name, p)) {
            return true;
        }
    }
    return false;
}

/// Estimate the size of a directory recursively
fn estimateDirSize(dir: std.fs.Dir, subpath: []const u8, allocator: std.mem.Allocator) !u64 {
    var size: u64 = 0;

    var subdir = try dir.openDir(subpath, .{ .iterate = true });
    defer subdir.close();

    var walker = try subdir.walk(allocator);
    defer walker.deinit();

    while (try walker.next()) |entry| {
        switch (entry.kind) {
            .file => {
                const stat = try entry.dir.statFile(entry.basename);
                size += stat.size;
            },
            .directory => {
                // Recursively estimate subdirectory size
                size += try estimateDirSize(entry.dir, entry.basename, allocator);
            },
            else => {},
        }
    }

    return size;
}
