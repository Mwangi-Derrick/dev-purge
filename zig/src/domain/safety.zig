const std = @import("std");

pub fn checkRootSafety(root_path: []const u8) !void {
    const dangerous_paths = [_][]const u8{
        "/", "/usr", "/etc", "/var", "/bin", "/sbin", "/lib",
        "C:\\Windows", "C:\\Program Files", "C:\\Program Files (x86)",
    };

    for (dangerous_paths) |dangerous| {
        if (std.mem.eql(u8, root_path, dangerous)) {
            std.debug.print("❌ Refusing to scan system directory: {s}\n", .{dangerous});
            return error.UnsafePath;
        }
    }

    if (std.mem.eql(u8, root_path, "~") or std.mem.endsWith(u8, root_path, "/home/") or std.mem.endsWith(u8, root_path, "\\Users\\")) {
        std.debug.print("❌ Refusing to scan home directory. Run from a specific project directory.\n", .{});
        return error.UnsafePath;
    }
}
