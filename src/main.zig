const std = @import("std");

pub fn main() !void {
    const args = std.os.argv[1..std.os.argv.len];

    const arg = args[0][0..8];
    const install_arg: []const u8 = "install";

    std.debug.print("{s}, {s}\n", .{ arg, install_arg });

    if (std.mem.eql(u8, arg, install_arg)) {
        std.debug.print("{s}\n", .{args[0]});
    }
}
