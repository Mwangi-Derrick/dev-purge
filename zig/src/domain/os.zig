const std = @import("std");
const builtin = @import("builtin");

pub const OsFamily = enum {
    Windows,
    Unix,
    Any,

    pub fn current() OsFamily {
        return switch (builtin.os.tag) {
            .windows => .Windows,
            else => .Unix,
        };
    }
};

pub const ProtectedCategory = enum {
    System,
    Ide,
    ToolBinary,
    ToolCache,
    Secret,
    Project,
};

pub const Rule = struct {
    category: ProtectedCategory,
    os: OsFamily,
    env_var: ?[]const u8 = null,
    sub_path: ?[]const u8 = null,
    name: []const u8,
    description: []const u8,
};

/// Compact rule definitions mirroring Rust version
pub const PROTECTED_RULES = [_]Rule{
    // System
    .{ .category = .System, .os = .Unix, .name = "bin", .description = "System binaries" },
    .{ .category = .System, .os = .Unix, .name = "usr", .description = "System user files" },
    .{ .category = .System, .os = .Unix, .name = "etc", .description = "System config" },
    .{ .category = .System, .os = .Windows, .name = "Windows", .description = "Windows system" },
    .{ .category = .System, .os = .Windows, .name = "Program Files", .description = "Installed apps" },

    // IDEs
    .{ .category = .Ide, .os = .Any, .name = ".vscode", .description = "VS Code configs" },
    .{ .category = .Ide, .os = .Any, .name = ".idea", .description = "JetBrains configs" },
    .{ .category = .Ide, .os = .Any, .name = ".cursor", .description = "Cursor configs" },

    // Version Control / Metadata
    .{ .category = .Project, .os = .Any, .name = ".git", .description = "Git repository" },
    .{ .category = .Project, .os = .Any, .name = ".github", .description = "GitHub workflows" },
    .{ .category = .Project, .os = .Any, .name = ".gitignore", .description = "Git ignore rules" },

    // Secrets
    .{ .category = .Secret, .os = .Any, .name = ".env", .description = "Environment variables" },
    .{ .category = .Secret, .os = .Any, .name = ".env.local", .description = "Local secrets" },

    // Tool Caches
    .{ .category = .ToolCache, .os = .Any, .name = ".cargo", .description = "Rust toolchain" },
    .{ .category = .ToolCache, .os = .Any, .name = ".npm", .description = "Node cache" },
    .{ .category = .ToolCache, .os = .Any, .name = ".gradle", .description = "Gradle cache" },
};

pub fn isProtectedEntry(name: []const u8) bool {
    const current_os = OsFamily.current();
    for (PROTECTED_RULES) |rule| {
        if (rule.os != .Any and rule.os != current_os) continue;
        if (std.mem.eql(u8, name, rule.name)) return true;
    }
    return false;
}
