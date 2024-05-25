const std = @import("std");
const vmp_versions = @import("versions.zig");

const io = std.io;
const print = std.debug.print;

pub fn install(gpa: std.mem.Allocator, version: []const u8) !void {
    print("installing version {s}\n", .{version});

    _ = try vmp_versions.fetchPythonVersions(gpa);
}
