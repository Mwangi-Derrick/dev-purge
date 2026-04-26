const std = @import("std");

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    const root_path = if (args.len > 1) args[1] else ".";

    std.debug.print("Scanning {s} for dev artifacts...\n", .{root_path});

    // Simple patterns to scan for
    const patterns = [_][]const u8{
        "target",
        "node_modules",
        "build",
        "dist",
        "__pycache__",
        ".venv",
        "venv",
    };

    var total_size: u64 = 0;
    var count: usize = 0;

    var dir = try std.fs.cwd().openDir(root_path, .{ .iterate = true });
    defer dir.close();

    var walker = try dir.walk(allocator);
    defer walker.deinit();

    while (try walker.next()) |entry| {
        if (entry.kind != .directory) continue;

        const name = std.fs.path.basename(entry.path);
        for (patterns) |pattern| {
            if (std.mem.eql(u8, name, pattern)) {
                // Estimate size (simplified)
                const size = try estimateDirSize(entry.dir, name, allocator);
                total_size += size;
                count += 1;
                std.debug.print("Found: {s} ({d} bytes)\n", .{ entry.path, size });
                break;
            }
        }
    }

    std.debug.print("Total: {d} items, {d} bytes recoverable\n", .{ count, total_size });
}

fn estimateDirSize(dir: std.fs.Dir, subpath: []const u8, allocator: std.mem.Allocator) !u64 {
    var size: u64 = 0;
    var walker = try dir.walk(allocator);
    defer walker.deinit();

    while (try walker.next()) |entry| {
        if (entry.kind == .file) {
            const stat = try entry.dir.statFile(entry.basename);
            size += stat.size;
        }
    }

    return size;
}
