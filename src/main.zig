const std = @import("std");
const clap = @import("clap");
const vmp_install = @import("install.zig");

const io = std.io;
const print = std.debug.print;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();

    const params = comptime clap.parseParamsComptime(
        \\-h, --help                        Display help and exit.
        \\-v, --version                     Display Version Number and exit.
        \\-i, --install <str>...            Install Python version
        \\-u, --use <str>...                Use Python version
        \\-r, --uninstall <str>...          Uninstall Python version
        \\-l, --list <str>...               List all available python versions
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
    for (res.args.install) |n| {
        const version = clap.parsers.string(n) catch |err| {
            print("{s}", .{err});
            return;
        };
        try vmp_install.install(gpa.allocator(), version);
    }
    for (res.args.use) |n|
        print("use = {s}\n", .{n});
    for (res.args.uninstall) |n|
        print("uninstall = {s}\n", .{n});
    for (res.args.list) |n|
        print("list = {s}\n", .{n});
}
