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

    if (std.mem.eql(u8, args[1], "--version") or std.mem.eql(u8, args[1], "-v")) {
        std.debug.print("dev-purge scout v0.1.0 (Zig 0.15.2)\n", .{});
        return;
    }

    const root_path = args[1];
    const dry_run = hasArg(args, "--dry-run");

    try safety.checkRootSafety(root_path);

    std.debug.print("DEV-PURGE (Zig): Systematically scanning {s}...\n", .{root_path});
    if (dry_run) {
        std.debug.print("Mode: Dry-run (Heuristic detection only)\n", .{});
    }

    var results = try std.ArrayList(scanner.ScanResult).initCapacity(allocator, 0);
    var root_dir = try std.fs.cwd().openDir(root_path, .{ .iterate = true });
    defer root_dir.close();

    try scanner.scan(root_dir, root_path, allocator, &results);

    const cleaner = @import("domain/cleaner.zig");
    const stats = try cleaner.purge(results.items, dry_run);

    const total_size_str = try formatSize(stats.total_bytes_freed, allocator);
    std.debug.print("\n✨ Reclamation complete! Purged {d} items. Total space: {s}\n", .{ stats.items_deleted, total_size_str });
    if (stats.errors > 0) std.debug.print("⚠️  Total errors encountered: {d}\n", .{stats.errors});
}

fn printUsage(exe_name: []const u8) void {
    std.debug.print("Usage: {s} <path> [--dry-run]\n", .{exe_name});
    std.debug.print("\nOptions:\n", .{});
    std.debug.print("  --dry-run    Preview purge candidates without execution\n", .{});
}

fn hasArg(args: []const [:0]u8, target: []const u8) bool {
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
