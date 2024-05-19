const std = @import("std");

const io = std.io;
const print = std.debug.print;
const json = std.json;

const Header = std.http.Client.Request.Headers;
const ResponseStorage = std.http.Client.FetchOptions.ResponseStorage;
const FetchOptions = std.http.Client.FetchOptions;
const Location = std.http.Client.FetchOptions.Location;

pub fn fetchPythonVersions() !void {
    const allocator = std.heap.page_allocator;
    var client = std.http.Client{
        .allocator = allocator,
    };

    const pythonGitTagsURI = try std.Uri.parse("https://api.github.com/repos/python/cpython/git/matching-refs/tags/v");

    const headers = Header{};

    var response_raw = std.ArrayList(u8).init(allocator);
    defer response_raw.deinit();

    const fetchOptions = FetchOptions{
        .method = .GET,
        .headers = headers,
        .location = Location{ .uri = pythonGitTagsURI },
        .response_storage = ResponseStorage{ .dynamic = &response_raw },
    };

    const req = try client.fetch(fetchOptions);

    print("{any}\n", .{req.status});

    const parseOpts = .{ .ignore_unknown_fields = true };
    const data = try json.parseFromSliceLeaky(VersionResponse, allocator, response_raw.items, parseOpts);

    print("{any}", .{data.len});

    for (data) |item| {
        const out: []u8 = try allocator.alloc(u8, item.ref.len - 10);
        _ = std.mem.replace(u8, item.ref, "refs/tags/", "", out);
        print("{s}\n", .{out});
    }
}

const VersionResponse = []struct {
    ref: []const u8,
};
