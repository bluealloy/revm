(module
  (type (;0;) (func (param i32 i32)))
  (type (;1;) (func (param i32)))
  (type (;2;) (func))
  (import "fluentbase_v1alpha" "_sys_write" (func $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2 (type 0)))
  (import "fluentbase_v1alpha" "_sys_halt" (func $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5 (type 1)))
  (func $main (type 2)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 20
    i32.add
    i64.const 0
    i64.store align=4
    local.get 0
    i32.const 1
    i32.store offset=12
    local.get 0
    i32.const 65552
    i32.store offset=8
    local.get 0
    i32.const 65560
    i32.store offset=16
    local.get 0
    i32.const 8
    i32.add
    call $core::panicking::panic_fmt::h78607b33a29a727d
    unreachable)
  (func $core::panicking::panic_fmt::h78607b33a29a727d (type 1) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    i32.const 65560
    call $_$LT$T$u20$as$u20$core..any..Any$GT$::type_id::hed637ffe26dba6a3
    block  ;; label = @1
      local.get 1
      i64.load
      i64.const -4493808902380553279
      i64.xor
      local.get 1
      i32.const 8
      i32.add
      i64.load
      i64.const -163230743173927068
      i64.xor
      i64.or
      i64.const 0
      i64.ne
      br_if 0 (;@1;)
      local.get 1
      local.get 1
      call $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2
    end
    i32.const -71
    call $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5
    unreachable)
  (func $_$LT$T$u20$as$u20$core..any..Any$GT$::type_id::hed637ffe26dba6a3 (type 0) (param i32 i32)
    local.get 0
    i64.const 568815540544143123
    i64.store offset=8
    local.get 0
    i64.const 5657071353825360256
    i64.store)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 65536))
  (export "memory" (memory 0))
  (export "main" (func $main))
  (data $.rodata (i32.const 65536) "it is panic time\00\00\01\00\10\00\00\00"))
