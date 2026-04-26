const std = @import("std");
const config = @import("config.zig");
const os_domain = @import("os.zig");

pub const ScanResult = struct {
    path: []const u8,
    size: u64,
};

pub fn scan(dir: std.fs.Dir, path: []const u8, allocator: std.mem.Allocator, results: *std.ArrayList(ScanResult)) !void {
    var iterable_dir = try dir.openDir(path, .{ .iterate = true });
    defer iterable_dir.close();

    var iter = iterable_dir.iterate();
    while (try iter.next()) |entry| {
        if (entry.kind != .directory) continue;

        const name = entry.name;

        // Skip protected paths (IDE, VCS, etc.)
        if (os_domain.isProtectedEntry(name)) continue;

        const full_path = try std.fs.path.join(allocator, &.{ path, name });

        if (config.matchesPattern(iterable_dir, name)) {
            const size = try estimateDirSize(iterable_dir, name, allocator);
            // Add allocator as first parameter to append
            try results.append(allocator, .{ .path = full_path, .size = size });
            // Do NOT recurse into matched artifacts
        } else {
            // Recurse into unknown directories
            try scan(iterable_dir, name, allocator, results);
        }
    }
}

pub fn estimateDirSize(dir: std.fs.Dir, subpath: []const u8, allocator: std.mem.Allocator) !u64 {
    var size: u64 = 0;
    var subdir = dir.openDir(subpath, .{ .iterate = true }) catch return 0;
    defer subdir.close();

    var walker = subdir.walk(allocator) catch return 0;
    defer walker.deinit();

    while (walker.next() catch null) |entry| {
        if (entry.kind == .file) {
            const stat = entry.dir.statFile(entry.basename) catch continue;
            size += stat.size;
        }
    }
    return size;
}