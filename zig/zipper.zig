const std = @import("std");

var heap_buffer: [64 * 1024 * 1024]u8 = undefined;
var fba = std.heap.FixedBufferAllocator.init(&heap_buffer);
const allocator = fba.allocator();

const ZipperList = struct {
    left: std.ArrayList(i32),
    right: std.ArrayList(i32),

    pub fn init() ZipperList {
        return .{
            .left = .{}, 
            .right = .{},
        };
    }

    pub fn deinit(self: *ZipperList) void {
        self.left.deinit(allocator);
        self.right.deinit(allocator);
    }

    pub fn insert(self: *ZipperList, index: usize, value: i32) !void {
        const current_pos = self.left.items.len;

        if (index < current_pos) {
            // --- 左にある -> 差分を右へ移動 ---
            const count = current_pos - index;
            
            // 1. まとめて容量確保（ここで1回だけアロケーションチェック）
            try self.right.ensureUnusedCapacity(allocator, count);

            // 2. スライスとして直接アクセスし、逆順でコピー
            const src_slice = self.left.items[index..];
            var k: usize = src_slice.len;
            while (k > 0) : (k -= 1) {
                // 安全チェックなしで追加（高速）
                self.right.appendAssumeCapacity(src_slice[k-1]);
            }

            // 3. 左側のサイズを一気に切り詰める
            self.left.shrinkRetainingCapacity(index);

        } else if (index > current_pos) {
            // --- 右にある -> 差分を左へ移動 ---
            const count = index - current_pos;

            // 1. まとめて容量確保
            try self.left.ensureUnusedCapacity(allocator, count);

            // 2. 右スタックの末尾(top)から必要な分だけ取り出して左へ
            // 右スタックは「逆順」なので、末尾からpopして左へappendすれば順序が直る
            const right_len = self.right.items.len;
            // 安全策：要求数が在庫を超えていないか（ロジック上ありえないが）
            const move_count = @min(count, right_len);
            
            const start_index = right_len - move_count;
            const src_slice = self.right.items[start_index..];

            // 右の末尾(top)から順に左へ移す（=スライスの後ろから走査）
            var k: usize = src_slice.len;
            while (k > 0) : (k -= 1) {
                self.left.appendAssumeCapacity(src_slice[k-1]);
            }

            // 3. 右側のサイズを一気に切り詰める
            self.right.shrinkRetainingCapacity(start_index);
        }
        
        // 挿入
        try self.left.append(allocator, value);
    }
};

export fn run_zig_zipper(iterations: i32) i32 {
    fba.reset();
    var zipper = ZipperList.init();
    defer zipper.deinit();

    var seed: usize = 123456789;
    var len: usize = 0;
    var i: i32 = 0;

    while (i < iterations) : (i += 1) {
        const pos = if (len == 0) 0 else seed % len;
        zipper.insert(pos, i) catch return -1;
        seed = (seed *% 1103515245 +% 12345) & 0x7fffffff;
        len += 1;
    }

    return 0;
}