const std = @import("std");

pub const PatternKind = enum {
    Exact,
    Prefix,
    Guarded,
};

pub const Category = enum {
    Core,
    Python,
    Node,
    Go,
    Java,
    DotNet,
    Mobile,
    Infra,
    Other,
};

pub const Rule = struct {
    category: Category,
    kind: PatternKind,
    name: []const u8,
    guard: ?[]const u8 = null,
    description: []const u8,
};

/// DX-centric Artifact Registry for Zig
pub const ARTIFACT_REGISTRY = [_]Rule{
    // Core
    .{ .category = .Core, .kind = .Exact, .name = "target", .description = "Rust build artifacts" },
    .{ .category = .Core, .kind = .Exact, .name = "dist", .description = "Distribution folder" },
    .{ .category = .Core, .kind = .Exact, .name = "build", .description = "Build artifacts" },
    .{ .category = .Core, .kind = .Exact, .name = "out", .description = "Output directory" },

    // Python
    .{ .category = .Python, .kind = .Exact, .name = "__pycache__", .description = "Python bytecode" },
    .{ .category = .Python, .kind = .Exact, .name = ".venv", .description = "Virtual environment" },
    .{ .category = .Python, .kind = .Exact, .name = "venv", .description = "Virtual environment" },

    // Node.js
    .{ .category = .Node, .kind = .Exact, .name = "node_modules", .description = "Node dependencies" },
    .{ .category = .Node, .kind = .Exact, .name = ".next", .description = "Next.js build" },
    .{ .category = .Node, .kind = .Exact, .name = ".vite", .description = "Vite build" },

    // .NET (Guarded)
    .{ .category = .DotNet, .kind = .Guarded, .name = "bin", .guard = ".csproj", .description = ".NET binary output" },
    .{ .category = .DotNet, .kind = .Guarded, .name = "obj", .guard = ".csproj", .description = ".NET intermediate" },

    // Prefixes
    .{ .category = .Other, .kind = .Prefix, .name = "cmake-build-", .description = "CMake build directory" },
};

pub fn matchesPattern(dir: std.fs.Dir, name: []const u8) bool {
    for (ARTIFACT_REGISTRY) |rule| {
        switch (rule.kind) {
            .Exact => if (std.mem.eql(u8, name, rule.name)) return true,
            .Prefix => if (std.mem.startsWith(u8, name, rule.name)) return true,
            .Guarded => {
                if (!std.mem.eql(u8, name, rule.name)) continue;
                if (rule.guard) |guard| {
                    if (std.mem.startsWith(u8, guard, ".")) {
                        var iter = dir.iterate();
                        while (iter.next() catch null) |entry| {
                            if (std.mem.endsWith(u8, entry.name, guard)) return true;
                        }
                    } else {
                        dir.access(guard, .{}) catch continue;
                        return true;
                    }
                }
            },
        }
    }
    return false;
}
