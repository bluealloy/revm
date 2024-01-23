(module
  (type (;0;) (func (param i32 i32)))
  (type (;1;) (func (param i32)))
  (type (;2;) (func (param i32 i32 i32)))
  (type (;3;) (func (param i32 i32 i32 i32)))
  (type (;4;) (func (param i32 i32 i32) (result i32)))
  (type (;5;) (func))
  (import "fluentbase_v1alpha" "_sys_write" (func $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2 (type 0)))
  (import "fluentbase_v1alpha" "_sys_halt" (func $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5 (type 1)))
  (import "fluentbase_v1alpha" "_sys_read" (func $fluentbase_sdk::bindings::_sys_read::he28ad825855b6201 (type 2)))
  (import "fluentbase_v1alpha" "_crypto_ecrecover" (func $fluentbase_sdk::bindings::_crypto_ecrecover::h8097a8b34f341435 (type 3)))
  (func $compiler_builtins::mem::memcmp::h934ee432a6c6c000 (type 4) (param i32 i32 i32) (result i32)
    (local i32 i32 i32)
    i32.const 0
    local.set 3
    block  ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      block  ;; label = @2
        loop  ;; label = @3
          local.get 0
          i32.load8_u
          local.tee 4
          local.get 1
          i32.load8_u
          local.tee 5
          i32.ne
          br_if 1 (;@2;)
          local.get 0
          i32.const 1
          i32.add
          local.set 0
          local.get 1
          i32.const 1
          i32.add
          local.set 1
          local.get 2
          i32.const -1
          i32.add
          local.tee 2
          i32.eqz
          br_if 2 (;@1;)
          br 0 (;@3;)
        end
      end
      local.get 4
      local.get 5
      i32.sub
      local.set 3
    end
    local.get 3)
  (func $memcmp (type 4) (param i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    local.get 2
    call $compiler_builtins::mem::memcmp::h934ee432a6c6c000)
  (func $main (type 5)
    (local i32)
    global.get $__stack_pointer
    i32.const 256
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 24
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i32.const 16
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i32.const 8
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i64.const 0
    i64.store
    local.get 0
    i32.const 0
    i32.const 32
    call $fluentbase_sdk::bindings::_sys_read::he28ad825855b6201
    local.get 0
    i32.const 88
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i32.const 80
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i32.const 72
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i32.const 32
    i32.add
    i32.const 32
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i32.const 32
    i32.add
    i32.const 24
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i32.const 32
    i32.add
    i32.const 16
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i32.const 32
    i32.add
    i32.const 8
    i32.add
    i64.const 0
    i64.store
    local.get 0
    i64.const 0
    i64.store offset=32
    local.get 0
    i32.const 32
    i32.add
    i32.const 32
    i32.const 64
    call $fluentbase_sdk::bindings::_sys_read::he28ad825855b6201
    local.get 0
    i32.const 0
    i32.store8 offset=101
    local.get 0
    i32.const 101
    i32.add
    i32.const 96
    i32.const 1
    call $fluentbase_sdk::bindings::_sys_read::he28ad825855b6201
    local.get 0
    i32.const 102
    i32.add
    i32.const 0
    i32.const 65
    memory.fill
    local.get 0
    i32.const 102
    i32.add
    i32.const 97
    i32.const 65
    call $fluentbase_sdk::bindings::_sys_read::he28ad825855b6201
    local.get 0
    i32.const 167
    i32.add
    i32.const 0
    i32.const 65
    memory.fill
    local.get 0
    local.get 0
    i32.const 32
    i32.add
    local.get 0
    i32.const 167
    i32.add
    local.get 0
    i32.load8_u offset=101
    call $fluentbase_sdk::bindings::_crypto_ecrecover::h8097a8b34f341435
    block  ;; label = @1
      local.get 0
      i32.const 102
      i32.add
      local.get 0
      i32.const 167
      i32.add
      i32.const 65
      call $memcmp
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.const 244
      i32.add
      i64.const 0
      i64.store align=4
      local.get 0
      i32.const 1
      i32.store offset=236
      local.get 0
      i32.const 65556
      i32.store offset=232
      local.get 0
      i32.const 65564
      i32.store offset=240
      local.get 0
      i32.const 232
      i32.add
      call $core::panicking::panic_fmt::h78607b33a29a727d
      unreachable
    end
    local.get 0
    i32.const 256
    i32.add
    global.set $__stack_pointer)
  (func $core::panicking::panic_fmt::h78607b33a29a727d (type 1) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    i32.const 65564
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
  (data $.rodata (i32.const 65536) "verification failed\00\00\00\01\00\13\00\00\00"))
