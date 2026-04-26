const std = @import("std");
const os_domain = @import("domain/os.zig");
const config = @import("domain/config.zig");
const scanner = @import("domain/scanner.zig");
const safety = @import("domain/safety.zig");

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2 or std.mem.eql(u8, args[1], "--help") or std.mem.eql(u8, args[1], "-h")) {
        printUsage(args[0]);
        return;
    }

    const root_path = args[1];
    const dry_run = hasArg(args, "--dry-run");

    try safety.checkRootSafety(root_path);

    std.debug.print("🛡️  DEV-PURGE (Zig): Scanning {s}...\n", .{root_path});
    if (dry_run) {
        std.debug.print("🔍 Dry-run mode active\n", .{});
    }

    var total_size: u64 = 0;
    var count: usize = 0;
    var errors: usize = 0;

    var dir = std.fs.cwd().openDir(root_path, .{ .iterate = true }) catch |err| {
        std.debug.print("❌ Failed to open directory {s}: {}\n", .{ root_path, err });
        return err;
    };
    defer dir.close();

    var walker = try dir.walk(allocator);
    defer walker.deinit();

    while (try walker.next()) |entry| {
        if (entry.kind != .directory) continue;

        const name = std.fs.path.basename(entry.path);

        if (os_domain.isProtectedEntry(name)) continue;

        if (config.matchesPattern(entry.dir, name)) {
            const size = scanner.estimateDirSize(entry.dir, name, allocator) catch |err| {
                std.debug.print("⚠️  Error estimating size for {s}: {}\n", .{ entry.path, err });
                errors += 1;
                continue;
            };

            total_size += size;
            count += 1;

            if (dry_run) {
                const size_str = try formatSize(size, allocator);
                std.debug.print("[DRY RUN] {s} ({s})\n", .{ entry.path, size_str });
            } else {
                const size_str = try formatSize(size, allocator);
                std.debug.print("🗑️  Deleting {s} ({s})\n", .{ entry.path, size_str });
                entry.dir.deleteTree(name) catch |err| {
                    std.debug.print("❌ Failed to delete {s}: {}\n", .{ entry.path, err });
                    errors += 1;
                    continue;
                };
            }
        }
    }

    const total_size_str = try formatSize(total_size, allocator);
    std.debug.print("\n✨ Done! Found {d} items. Total recoverable: {s}\n", .{ count, total_size_str });
    if (errors > 0) std.debug.print("⚠️  Total errors: {d}\n", .{errors});
}

fn printUsage(exe_name: []const u8) void {
    std.debug.print("Usage: {s} <path> [--dry-run]\n", .{exe_name});
    std.debug.print("\nOptions:\n", .{});
    std.debug.print("  --dry-run    Show what would be deleted without actually deleting\n", .{});
}

fn hasArg(args: [][]const u8, target: []const u8) bool {
    for (args) |arg| {
        if (std.mem.eql(u8, arg, target)) return true;
    }
    return false;
}

fn formatSize(bytes: u64, allocator: std.mem.Allocator) ![]u8 {
    if (bytes < 1024) {
        return try std.fmt.allocPrint(allocator, "{d} B", .{bytes});
    } else if (bytes < 1024 * 1024) {
        return try std.fmt.allocPrint(allocator, "{d:.2} KB", .{@as(f64, @floatFromInt(bytes)) / 1024.0});
    } else if (bytes < 1024 * 1024 * 1024) {
        return try std.fmt.allocPrint(allocator, "{d:.2} MB", .{@as(f64, @floatFromInt(bytes)) / (1024.0 * 1024.0)});
    } else {
        return try std.fmt.allocPrint(allocator, "{d:.2} GB", .{@as(f64, @floatFromInt(bytes)) / (1024.0 * 1024.0 * 1024.0)});
    }
}
