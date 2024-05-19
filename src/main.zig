const std = @import("std");
const clap = @import("clap");

const io = std.io;
const print = std.debug.print;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();

    const params = comptime clap.parseParamsComptime(
        \\-h, --help                        Display this help and exit.
        \\-i, --install <str>...            Install
        \\-u, --use <str>...
        \\-r, --uninstall <str>...
        \\<str>...
        \\
    );

    var diag = clap.Diagnostic{};
    var res = clap.parse(clap.Help, &params, clap.parsers.default, .{
        .diagnostic = &diag,
        .allocator = gpa.allocator(),
    }) catch |err| {
        // Report useful error and exit
        diag.report(io.getStdErr().writer(), err) catch {};
        return err;
    };
    defer res.deinit();

    if (res.args.help != 0)
        print("--help\n", .{});
    for (res.args.install) |n|
        print("install = {s}\n", .{n});
    for (res.args.use) |n|
        print("install = {s}\n", .{n});
    for (res.args.install) |n|
        print("install = {s}\n", .{n});
}
