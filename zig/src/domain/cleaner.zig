const std = @import("std");
const scanner = @import("scanner.zig");

pub const CleanupStats = struct {
    total_bytes_freed: u64,
    items_deleted: usize,
    errors: usize,
};

pub fn purge(results: []const scanner.ScanResult, dry_run: bool) !CleanupStats {
    var stats = CleanupStats{
        .total_bytes_freed = 0,
        .items_deleted = 0,
        .errors = 0,
    };

    for (results) |res| {
        if (dry_run) {
            std.debug.print("[DRY RUN] Would purge: {s} ({d} bytes)\n", .{ res.path, res.size });
            stats.total_bytes_freed += res.size;
            stats.items_deleted += 1;
        } else {
            // Robust deletion for "Senior Engineer" standards
            std.debug.print("🗑️  Purging: {s}... ", .{res.path});
            
            // On Windows, we might need to handle read-only attributes or long paths.
            // std.fs.cwd().deleteTree(res.path) is generally good, but we catch specific errors.
            std.fs.cwd().deleteTree(res.path) catch |err| {
                if (err == error.AccessDenied) {
                    std.debug.print("❌ Access Denied (is it in use?)\n", .{});
                } else if (err == error.FileNotFound) {
                    std.debug.print("❓ Already gone\n", .{});
                } else {
                    std.debug.print("❌ Failed: {}\n", .{err});
                }
                stats.errors += 1;
                continue;
            };

            std.debug.print("✓\n", .{});
            stats.total_bytes_freed += res.size;
            stats.items_deleted += 1;
        }
    }

    return stats;
}
