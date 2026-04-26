const std = @import("std");

pub fn estimateDirSize(dir: std.fs.Dir, subpath: []const u8, allocator: std.mem.Allocator) !u64 {
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
            else => {},
        }
    }
    return size;
}
