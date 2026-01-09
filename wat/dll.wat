(module
  ;; メモリ定義 (1ページ = 64KB)
  (memory $mem 256)
  (export "memory" (memory $mem))

  ;; ==========================================
  ;; 定数定義
  ;; ==========================================
  (global $ADDR_HEAD (mut i32) (i32.const 0))
  (global $ADDR_TAIL (mut i32) (i32.const 0))
  (global $ADDR_ALLOC (mut i32) (i32.const 12)) ;; 12バイト目からヒープ開始

  (global $NODE_SIZE i32 (i32.const 12))
  (global $OFF_PREV i32 (i32.const 0))
  (global $OFF_NEXT i32 (i32.const 4))
  (global $OFF_VAL  i32 (i32.const 8))

  ;; ==========================================
  ;; アロケータ (Bump Allocator)
  ;; ==========================================
  (func $alloc (result i32)
    (local $ptr i32)
    global.get $ADDR_ALLOC
    local.set $ptr
    global.get $ADDR_ALLOC
    global.get $NODE_SIZE
    i32.add
    global.set $ADDR_ALLOC
    local.get $ptr
  )

  ;; ==========================================
  ;; リセット (ベンチマーク毎にメモリを初期化)
  ;; ==========================================
  (func $reset
    i32.const 0
    global.set $ADDR_HEAD
    i32.const 0
    global.set $ADDR_TAIL
    i32.const 12
    global.set $ADDR_ALLOC
  )

  ;; ==========================================
  ;; Append (末尾追加)
  ;; ==========================================
  (func $append (param $value i32)
    (local $new_node i32)
    (local $old_tail i32)

    call $alloc
    local.set $new_node

    ;; new_node.val = value
    local.get $new_node
    global.get $OFF_VAL
    i32.add
    local.get $value
    i32.store

    ;; new_node.next = NULL
    local.get $new_node
    global.get $OFF_NEXT
    i32.add
    i32.const 0
    i32.store

    global.get $ADDR_TAIL
    local.set $old_tail

    ;; new_node.prev = old_tail
    local.get $new_node
    global.get $OFF_PREV
    i32.add
    local.get $old_tail
    i32.store

    ;; リストが空か判定
    local.get $old_tail
    i32.eqz
    (if
      (then
        ;; Head = new_node
        local.get $new_node
        global.set $ADDR_HEAD
      )
      (else
        ;; old_tail.next = new_node
        local.get $old_tail
        global.get $OFF_NEXT
        i32.add
        local.get $new_node
        i32.store
      )
    )
    
    ;; Tail更新
    local.get $new_node
    global.set $ADDR_TAIL
  )

  ;; ==========================================
  ;; Sum (合計計算・トラバーサル)
  ;; ==========================================
  (func $sum (result i32)
    (local $current i32)
    (local $acc i32)
    
    global.get $ADDR_HEAD
    local.set $current
    i32.const 0
    local.set $acc

    (block $break
      (loop $top
        local.get $current
        i32.eqz
        br_if $break

        ;; acc += current.val
        local.get $acc
        local.get $current
        global.get $OFF_VAL
        i32.add
        i32.load
        i32.add
        local.set $acc

        ;; current = current.next
        local.get $current
        global.get $OFF_NEXT
        i32.add
        i32.load
        local.set $current

        br $top
      )
    )
    local.get $acc
  )

  ;; ==========================================
  ;; 【重要】メイン実行関数 (JSから呼ばれる)
  ;; ==========================================
  (func $run_wat_dll (export "run_wat_dll") (param $iters i32) (result i32)
    (local $i i32)
    
    ;; 1. 状態をリセット
    call $reset

    ;; 2. 指定回数 append を繰り返す
    (block $break
      (loop $top
        local.get $i
        local.get $iters
        i32.ge_s
        br_if $break

        local.get $i
        call $append

        local.get $i
        i32.const 1
        i32.add
        local.set $i
        br $top
      )
    )
    
    ;; 3. 合計を計算して返す
    call $sum
  )
)