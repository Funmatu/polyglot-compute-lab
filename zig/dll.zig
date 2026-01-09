const std = @import("std");

// OSにメモリを要求せず、あらかじめ確保した静的な配列をヒープとして使う。
// 64MBのバッファを確保（WASMメモリ空間内に配置される）
var heap_buffer: [64 * 1024 * 1024]u8 = undefined;

// 固定バッファアロケータを初期化
var fba = std.heap.FixedBufferAllocator.init(&heap_buffer);
const allocator = fba.allocator();

const Node = struct {
    value: i32,
    prev: ?*Node,
    next: ?*Node,
};

const DoublyLinkedList = struct {
    head: ?*Node,
    tail: ?*Node,

    pub fn init() DoublyLinkedList {
        return .{ .head = null, .tail = null };
    }

    pub fn append(self: *DoublyLinkedList, value: i32) !void {
        // 固定バッファからメモリを切り出す
        const new_node = try allocator.create(Node);
        new_node.* = .{
            .value = value,
            .prev = null,
            .next = null,
        };

        if (self.tail) |tail_node| {
            tail_node.next = new_node;
            new_node.prev = tail_node;
            self.tail = new_node;
        } else {
            self.head = new_node;
            self.tail = new_node;
        }
    }

    pub fn sum(self: *DoublyLinkedList) i32 {
        var s: i32 = 0;
        var current = self.head;
        while (current) |node| {
            s += node.value;
            current = node.next;
        }
        return s;
    }
};

// WASMから呼び出すためのエクスポート関数
export fn run_zig_dll(iterations: i32) i32 {
    // ベンチマークごとにアロケータをリセット（メモリを再利用）
    fba.reset();
    
    var dll = DoublyLinkedList.init();
    
    var i: i32 = 0;
    while (i < iterations) : (i += 1) {
        // エラーハンドリング: メモリ不足などの場合
        // ここではベンチマークなので簡易的に -1 を返す
        dll.append(i) catch return -1;
    }
    
    return dll.sum();
}