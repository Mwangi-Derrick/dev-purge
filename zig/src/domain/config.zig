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
    // Core / Generic
    .{ .category = .Core, .kind = .Exact, .name = "target", .description = "Rust build artifacts" },
    .{ .category = .Core, .kind = .Exact, .name = "dist", .description = "Generic distribution folder" },
    .{ .category = .Core, .kind = .Exact, .name = "build", .description = "Generic build artifacts" },
    .{ .category = .Core, .kind = .Exact, .name = "out", .description = "Generic output directory" },

    // Python Ecosystem
    .{ .category = .Python, .kind = .Exact, .name = "__pycache__", .description = "Python bytecode cache" },
    .{ .category = .Python, .kind = .Exact, .name = ".venv", .description = "Python virtual environment" },
    .{ .category = .Python, .kind = .Exact, .name = "venv", .description = "Python virtual environment" },
    .{ .category = .Python, .kind = .Exact, .name = ".pytest_cache", .description = "Pytest execution cache" },
    .{ .category = .Python, .kind = .Exact, .name = ".mypy_cache", .description = "Mypy type check cache" },

    // Web / JavaScript / TypeScript
    .{ .category = .Node, .kind = .Exact, .name = "node_modules", .description = "Node.js dependencies" },
    .{ .category = .Node, .kind = .Exact, .name = ".next", .description = "Next.js build artifacts" },
    .{ .category = .Node, .kind = .Exact, .name = ".nuxt", .description = "Nuxt.js build artifacts" },
    .{ .category = .Node, .kind = .Exact, .name = ".turbo", .description = "Turborepo build cache" },
    .{ .category = .Node, .kind = .Exact, .name = ".vite", .description = "Vite build cache" },

    // Go / PHP / Ruby
    .{ .category = .Go, .kind = .Exact, .name = "vendor", .description = "Dependency vendor directory" },

    // Java / Kotlin / Gradle
    .{ .category = .Java, .kind = .Exact, .name = ".gradle", .description = "Gradle build cache" },
    .{ .category = .Java, .kind = .Exact, .name = ".kotlin", .description = "Kotlin compiler metadata" },

    // .NET / C# (Heuristic-based)
    .{ .category = .DotNet, .kind = .Guarded, .name = "bin", .guard = ".csproj", .description = ".NET binary output" },
    .{ .category = .DotNet, .kind = .Guarded, .name = "obj", .guard = ".csproj", .description = ".NET intermediate artifacts" },
    .{ .category = .DotNet, .kind = .Guarded, .name = "bin", .guard = ".sln", .description = ".NET solution binaries" },
    .{ .category = .DotNet, .kind = .Guarded, .name = "obj", .guard = ".sln", .description = ".NET solution intermediates" },

    // Mobile & Cross-Platform
    .{ .category = .Mobile, .kind = .Exact, .name = ".dart_tool", .description = "Dart/Flutter metadata" },
    .{ .category = .Mobile, .kind = .Exact, .name = "DerivedData", .description = "Xcode build artifacts" },

    // Infrastructure & Tooling
    .{ .category = .Infra, .kind = .Exact, .name = ".terraform", .description = "Terraform state/plugins" },
    .{ .category = .Infra, .kind = .Exact, .name = "zig-cache", .description = "Zig build cache" },
    .{ .category = .Infra, .kind = .Exact, .name = "zig-out", .description = "Zig binary output" },

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
