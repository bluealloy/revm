(module
  (type (;0;) (func (param i32 i32 i32) (result i32)))
  (type (;1;) (func (param i32 i32) (result i32)))
  (type (;2;) (func (param i32 i32)))
  (type (;3;) (func (param i32)))
  (type (;4;) (func (param i32 i32 i32)))
  (type (;5;) (func))
  (type (;6;) (func (param i32 i32 i32 i32) (result i32)))
  (import "fluentbase_v1alpha" "_sys_write" (func $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2 (type 2)))
  (import "fluentbase_v1alpha" "_sys_halt" (func $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5 (type 3)))
  (func $compiler_builtins::mem::memcpy::h4ab6845275ba77f4 (type 0) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 2
        i32.const 16
        i32.ge_u
        br_if 0 (;@2;)
        local.get 0
        local.set 3
        br 1 (;@1;)
      end
      local.get 0
      i32.const 0
      local.get 0
      i32.sub
      i32.const 3
      i32.and
      local.tee 4
      i32.add
      local.set 5
      block  ;; label = @2
        local.get 4
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.set 3
        local.get 1
        local.set 6
        loop  ;; label = @3
          local.get 3
          local.get 6
          i32.load8_u
          i32.store8
          local.get 6
          i32.const 1
          i32.add
          local.set 6
          local.get 3
          i32.const 1
          i32.add
          local.tee 3
          local.get 5
          i32.lt_u
          br_if 0 (;@3;)
        end
      end
      local.get 5
      local.get 2
      local.get 4
      i32.sub
      local.tee 7
      i32.const -4
      i32.and
      local.tee 8
      i32.add
      local.set 3
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          local.get 4
          i32.add
          local.tee 9
          i32.const 3
          i32.and
          i32.eqz
          br_if 0 (;@3;)
          local.get 8
          i32.const 1
          i32.lt_s
          br_if 1 (;@2;)
          local.get 9
          i32.const 3
          i32.shl
          local.tee 6
          i32.const 24
          i32.and
          local.set 2
          local.get 9
          i32.const -4
          i32.and
          local.tee 10
          i32.const 4
          i32.add
          local.set 1
          i32.const 0
          local.get 6
          i32.sub
          i32.const 24
          i32.and
          local.set 4
          local.get 10
          i32.load
          local.set 6
          loop  ;; label = @4
            local.get 5
            local.get 6
            local.get 2
            i32.shr_u
            local.get 1
            i32.load
            local.tee 6
            local.get 4
            i32.shl
            i32.or
            i32.store
            local.get 1
            i32.const 4
            i32.add
            local.set 1
            local.get 5
            i32.const 4
            i32.add
            local.tee 5
            local.get 3
            i32.lt_u
            br_if 0 (;@4;)
            br 2 (;@2;)
          end
        end
        local.get 8
        i32.const 1
        i32.lt_s
        br_if 0 (;@2;)
        local.get 9
        local.set 1
        loop  ;; label = @3
          local.get 5
          local.get 1
          i32.load
          i32.store
          local.get 1
          i32.const 4
          i32.add
          local.set 1
          local.get 5
          i32.const 4
          i32.add
          local.tee 5
          local.get 3
          i32.lt_u
          br_if 0 (;@3;)
        end
      end
      local.get 7
      i32.const 3
      i32.and
      local.set 2
      local.get 9
      local.get 8
      i32.add
      local.set 1
    end
    block  ;; label = @1
      local.get 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 3
      local.get 2
      i32.add
      local.set 5
      loop  ;; label = @2
        local.get 3
        local.get 1
        i32.load8_u
        i32.store8
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 3
        i32.const 1
        i32.add
        local.tee 3
        local.get 5
        i32.lt_u
        br_if 0 (;@2;)
      end
    end
    local.get 0)
  (func $memcpy (type 0) (param i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    local.get 2
    call $compiler_builtins::mem::memcpy::h4ab6845275ba77f4)
  (func $_$LT$core..ops..range..Range$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$::index::h92056be543c3aae7 (type 4) (param i32 i32 i32)
    block  ;; label = @1
      local.get 1
      i32.const 1832
      i32.gt_u
      br_if 0 (;@1;)
      local.get 0
      local.get 1
      i32.store offset=4
      local.get 0
      local.get 2
      i32.store
      return
    end
    local.get 1
    call $core::slice::index::slice_end_index_len_fail::h6372e465cf26b33a
    unreachable)
  (func $core::slice::index::slice_end_index_len_fail::h6372e465cf26b33a (type 3) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    local.get 0
    i32.store
    local.get 1
    i32.const 1832
    i32.store offset=4
    local.get 1
    i32.const 8
    i32.add
    i32.const 12
    i32.add
    i64.const 2
    i64.store align=4
    local.get 1
    i32.const 32
    i32.add
    i32.const 12
    i32.add
    i32.const 1
    i32.store
    local.get 1
    i32.const 2
    i32.store offset=12
    local.get 1
    i32.const 67648
    i32.store offset=8
    local.get 1
    i32.const 1
    i32.store offset=36
    local.get 1
    local.get 1
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 1
    local.get 1
    i32.const 4
    i32.add
    i32.store offset=40
    local.get 1
    local.get 1
    i32.store offset=32
    local.get 1
    i32.const 8
    i32.add
    i32.const 67380
    call $core::panicking::panic_fmt::h78607b33a29a727d
    unreachable)
  (func $fluentbase_codec::encoder::Encoder::encode_to_fixed::h528422d67065c231 (type 3) (param i32)
    local.get 0
    i32.const 8
    i32.add
    i32.const 65536
    i32.const 1824
    call $memcpy
    drop
    local.get 0
    i32.const 1832
    i32.store offset=1832
    local.get 0
    i64.const 7834020347912
    i64.store align=4)
  (func $deploy (type 5)
    (local i32)
    global.get $__stack_pointer
    i32.const 3680
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 1844
    i32.add
    call $fluentbase_codec::encoder::Encoder::encode_to_fixed::h528422d67065c231
    local.get 0
    i32.const 12
    i32.add
    local.get 0
    i32.const 1844
    i32.add
    i32.const 1832
    call $memcpy
    drop
    local.get 0
    local.get 0
    i32.load offset=3676
    local.get 0
    i32.const 12
    i32.add
    call $_$LT$core..ops..range..Range$LT$usize$GT$$u20$as$u20$core..slice..index..SliceIndex$LT$$u5b$T$u5d$$GT$$GT$::index::h92056be543c3aae7
    local.get 0
    i32.load
    local.get 0
    i32.load offset=4
    call $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2
    i32.const 0
    call $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5
    unreachable)
  (func $core::panicking::panic_fmt::h78607b33a29a727d (type 2) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 67396
    call $_$LT$T$u20$as$u20$core..any..Any$GT$::type_id::hed637ffe26dba6a3
    block  ;; label = @1
      local.get 2
      i64.load
      i64.const -4493808902380553279
      i64.xor
      local.get 2
      i32.const 8
      i32.add
      i64.load
      i64.const -163230743173927068
      i64.xor
      i64.or
      i64.const 0
      i64.ne
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      call $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2
    end
    i32.const -71
    call $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5
    unreachable)
  (func $_$LT$T$u20$as$u20$core..any..Any$GT$::type_id::hed637ffe26dba6a3 (type 2) (param i32 i32)
    local.get 0
    i64.const 568815540544143123
    i64.store offset=8
    local.get 0
    i64.const 5657071353825360256
    i64.store)
  (func $core::fmt::num::imp::_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$::fmt::he696c0e431156bce (type 1) (param i32 i32) (result i32)
    (local i32 i32 i64 i64 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    i32.const 39
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i64.load32_u
        local.tee 4
        i64.const 10000
        i64.ge_u
        br_if 0 (;@2;)
        local.get 4
        local.set 5
        br 1 (;@1;)
      end
      i32.const 39
      local.set 3
      loop  ;; label = @2
        local.get 2
        i32.const 9
        i32.add
        local.get 3
        i32.add
        local.tee 0
        i32.const -4
        i32.add
        local.get 4
        i64.const 10000
        i64.div_u
        local.tee 5
        i64.const 55536
        i64.mul
        local.get 4
        i64.add
        i32.wrap_i64
        local.tee 6
        i32.const 65535
        i32.and
        i32.const 100
        i32.div_u
        local.tee 7
        i32.const 1
        i32.shl
        i32.const 67396
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        local.get 0
        i32.const -2
        i32.add
        local.get 7
        i32.const -100
        i32.mul
        local.get 6
        i32.add
        i32.const 65535
        i32.and
        i32.const 1
        i32.shl
        i32.const 67396
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        local.get 3
        i32.const -4
        i32.add
        local.set 3
        local.get 4
        i64.const 99999999
        i64.gt_u
        local.set 0
        local.get 5
        local.set 4
        local.get 0
        br_if 0 (;@2;)
      end
    end
    block  ;; label = @1
      local.get 5
      i32.wrap_i64
      local.tee 0
      i32.const 99
      i32.le_u
      br_if 0 (;@1;)
      local.get 2
      i32.const 9
      i32.add
      local.get 3
      i32.const -2
      i32.add
      local.tee 3
      i32.add
      local.get 5
      i32.wrap_i64
      local.tee 6
      i32.const 65535
      i32.and
      i32.const 100
      i32.div_u
      local.tee 0
      i32.const -100
      i32.mul
      local.get 6
      i32.add
      i32.const 65535
      i32.and
      i32.const 1
      i32.shl
      i32.const 67396
      i32.add
      i32.load16_u align=1
      i32.store16 align=1
    end
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.const 10
        i32.lt_u
        br_if 0 (;@2;)
        local.get 2
        i32.const 9
        i32.add
        local.get 3
        i32.const -2
        i32.add
        local.tee 3
        i32.add
        local.get 0
        i32.const 1
        i32.shl
        i32.const 67396
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        br 1 (;@1;)
      end
      local.get 2
      i32.const 9
      i32.add
      local.get 3
      i32.const -1
      i32.add
      local.tee 3
      i32.add
      local.get 0
      i32.const 48
      i32.add
      i32.store8
    end
    i32.const 39
    local.get 3
    i32.sub
    local.set 8
    i32.const 1
    local.set 7
    i32.const 43
    i32.const 1114112
    local.get 1
    i32.load offset=28
    local.tee 0
    i32.const 1
    i32.and
    local.tee 6
    select
    local.set 9
    local.get 0
    i32.const 29
    i32.shl
    i32.const 31
    i32.shr_s
    i32.const 67396
    i32.and
    local.set 10
    local.get 2
    i32.const 9
    i32.add
    local.get 3
    i32.add
    local.set 11
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.load
        br_if 0 (;@2;)
        local.get 1
        i32.load offset=20
        local.tee 3
        local.get 1
        i32.load offset=24
        local.tee 0
        local.get 9
        local.get 10
        call $core::fmt::Formatter::pad_integral::write_prefix::h43684999422d0638
        br_if 1 (;@1;)
        local.get 3
        local.get 11
        local.get 8
        local.get 0
        i32.load offset=12
        call_indirect (type 0)
        local.set 7
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 1
        i32.load offset=4
        local.tee 12
        local.get 6
        local.get 8
        i32.add
        local.tee 7
        i32.gt_u
        br_if 0 (;@2;)
        i32.const 1
        local.set 7
        local.get 1
        i32.load offset=20
        local.tee 3
        local.get 1
        i32.load offset=24
        local.tee 0
        local.get 9
        local.get 10
        call $core::fmt::Formatter::pad_integral::write_prefix::h43684999422d0638
        br_if 1 (;@1;)
        local.get 3
        local.get 11
        local.get 8
        local.get 0
        i32.load offset=12
        call_indirect (type 0)
        local.set 7
        br 1 (;@1;)
      end
      block  ;; label = @2
        local.get 0
        i32.const 8
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.load offset=16
        local.set 13
        local.get 1
        i32.const 48
        i32.store offset=16
        local.get 1
        i32.load8_u offset=32
        local.set 14
        i32.const 1
        local.set 7
        local.get 1
        i32.const 1
        i32.store8 offset=32
        local.get 1
        i32.load offset=20
        local.tee 0
        local.get 1
        i32.load offset=24
        local.tee 15
        local.get 9
        local.get 10
        call $core::fmt::Formatter::pad_integral::write_prefix::h43684999422d0638
        br_if 1 (;@1;)
        local.get 3
        local.get 12
        i32.add
        local.get 6
        i32.sub
        i32.const -38
        i32.add
        local.set 3
        block  ;; label = @3
          loop  ;; label = @4
            local.get 3
            i32.const -1
            i32.add
            local.tee 3
            i32.eqz
            br_if 1 (;@3;)
            local.get 0
            i32.const 48
            local.get 15
            i32.load offset=16
            call_indirect (type 1)
            i32.eqz
            br_if 0 (;@4;)
            br 3 (;@1;)
          end
        end
        local.get 0
        local.get 11
        local.get 8
        local.get 15
        i32.load offset=12
        call_indirect (type 0)
        br_if 1 (;@1;)
        local.get 1
        local.get 14
        i32.store8 offset=32
        local.get 1
        local.get 13
        i32.store offset=16
        i32.const 0
        local.set 7
        br 1 (;@1;)
      end
      local.get 12
      local.get 7
      i32.sub
      local.set 12
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.load8_u offset=32
            local.tee 3
            br_table 2 (;@2;) 0 (;@4;) 1 (;@3;) 0 (;@4;) 2 (;@2;)
          end
          local.get 12
          local.set 3
          i32.const 0
          local.set 12
          br 1 (;@2;)
        end
        local.get 12
        i32.const 1
        i32.shr_u
        local.set 3
        local.get 12
        i32.const 1
        i32.add
        i32.const 1
        i32.shr_u
        local.set 12
      end
      local.get 3
      i32.const 1
      i32.add
      local.set 3
      local.get 1
      i32.const 24
      i32.add
      i32.load
      local.set 0
      local.get 1
      i32.load offset=16
      local.set 15
      local.get 1
      i32.load offset=20
      local.set 6
      block  ;; label = @2
        loop  ;; label = @3
          local.get 3
          i32.const -1
          i32.add
          local.tee 3
          i32.eqz
          br_if 1 (;@2;)
          local.get 6
          local.get 15
          local.get 0
          i32.load offset=16
          call_indirect (type 1)
          i32.eqz
          br_if 0 (;@3;)
        end
        i32.const 1
        local.set 7
        br 1 (;@1;)
      end
      i32.const 1
      local.set 7
      local.get 6
      local.get 0
      local.get 9
      local.get 10
      call $core::fmt::Formatter::pad_integral::write_prefix::h43684999422d0638
      br_if 0 (;@1;)
      local.get 6
      local.get 11
      local.get 8
      local.get 0
      i32.load offset=12
      call_indirect (type 0)
      br_if 0 (;@1;)
      i32.const 0
      local.set 3
      loop  ;; label = @2
        block  ;; label = @3
          local.get 12
          local.get 3
          i32.ne
          br_if 0 (;@3;)
          local.get 12
          local.get 12
          i32.lt_u
          local.set 7
          br 2 (;@1;)
        end
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        local.get 6
        local.get 15
        local.get 0
        i32.load offset=16
        call_indirect (type 1)
        i32.eqz
        br_if 0 (;@2;)
      end
      local.get 3
      i32.const -1
      i32.add
      local.get 12
      i32.lt_u
      local.set 7
    end
    local.get 2
    i32.const 48
    i32.add
    global.set $__stack_pointer
    local.get 7)
  (func $core::fmt::Formatter::pad_integral::write_prefix::h43684999422d0638 (type 6) (param i32 i32 i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 2
          i32.const 1114112
          i32.eq
          br_if 0 (;@3;)
          i32.const 1
          local.set 4
          local.get 0
          local.get 2
          local.get 1
          i32.load offset=16
          call_indirect (type 1)
          br_if 1 (;@2;)
        end
        local.get 3
        br_if 1 (;@1;)
        i32.const 0
        local.set 4
      end
      local.get 4
      return
    end
    local.get 0
    local.get 3
    i32.const 0
    local.get 1
    i32.load offset=12
    call_indirect (type 0))
  (table (;0;) 2 2 funcref)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 65536))
  (export "memory" (memory 0))
  (export "deploy" (func $deploy))
  (elem (;0;) (i32.const 1) func $core::fmt::num::imp::_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$::fmt::he696c0e431156bce)
  (data $.rodata (i32.const 65536) "\00asm\01\00\00\00\01\0d\03`\02\7f\7f\00`\01\7f\00`\00\00\02@\02\12fluentbase_v1alpha\0a_sys_write\00\00\12fluentbase_v1alpha\09_sys_halt\00\01\03\04\03\02\01\00\05\03\01\00\02\06\08\01\7f\01A\80\80\04\0b\07\11\02\06memory\02\00\04main\00\02\0a\d0\01\03H\01\01\7f#\80\80\80\80\00A k\22\00$\80\80\80\80\00 \00A\14jB\007\02\00 \00A\016\02\0c \00A\90\80\84\80\006\02\08 \00A\98\80\84\80\006\02\10 \00A\08j\10\83\80\80\80\00\00\0bc\01\01\7f#\80\80\80\80\00A\10k\22\01$\80\80\80\80\00 \01A\98\80\84\80\00\10\84\80\80\80\00\02@ \01)\03\00B\c1\f7\f9\e8\cc\93\b2\d1A\85 \01A\08j)\03\00B\e4\de\c7\85\90\d0\85\de}\85\84B\00R\0d\00 \01 \01\10\80\80\80\80\00\0bA\b9\7f\10\81\80\80\80\00\00\0b!\00 \00B\93\fe\eb\e6\86\d7\b5\f2\077\03\08 \00B\80\f3\b1\8e\c8\8d\fd\c0\ce\007\03\00\0b\0b \01\00A\80\80\04\0b\18it is panic time\00\00\01\00\10\00\00\00\00Y\0d.debug_abbrev\01\11\01%\0e\13\05\03\0e\10\17\1b\0e\b4B\19\11\01U\17\00\00\029\01\03\0e\00\00\03.\00\11\01\12\06@\18n\0e\03\0e:\0b;\0b6\0b?\19\87\01\19\00\00\04.\00\11\01\12\06@\18n\0e\03\0e:\0b;\0b\00\00\00\00\7f\0b.debug_infoo\00\00\00\04\00\00\00\00\00\04\01O\01\00\00\1c\00\19\01\00\00\00\00\00\00!\00\00\00\00\00\00\00\00\00\00\00\02Q\00\00\00\02\17\00\00\00\03K\00\00\00c\00\00\00\04\ed\00\01\9fV\00\00\00\0d\00\00\00\014\03\00\02\09\00\00\00\02\00\00\00\00\04\af\00\00\00!\00\00\00\07\ed\03\00\00\00\00\9f\87\00\00\00\cd\00\00\00\02\88\00\00\00\00\00&\0d.debug_rangesK\00\00\00\ae\00\00\00\af\00\00\00\d0\00\00\00\00\00\00\00\00\00\00\00\00\9b\03\0a.debug_str{impl#0}\00any\00panic_fmt\00panicking\00/rustc/4b85902b438f791c5bfcb6b1c5b476d5b88e2bef\00core\00_ZN4core9panicking9panic_fmt17h78607b33a29a727dE\00_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17hed637ffe26dba6a3E\00type_id<core::panic::panic_info::{impl#0}::internal_constructor::NoPayload>\00library/core/src/lib.rs/@/core.c753f00eab489041-cgu.0\00clang LLVM (rustc version 1.75.0-nightly (4b85902b4 2023-11-04))\00\00\ac\01\0f.debug_pubnames\98\00\00\00\02\00\00\00\00\00s\00\00\00&\00\00\00core\00+\00\00\00panicking\000\00\00\00panic_fmt\00J\00\00\00any\00O\00\00\00{impl#0}\00T\00\00\00type_id<core::panic::panic_info::{impl#0}::internal_constructor::NoPayload>\00\00\00\00\00\00\22\0f.debug_pubtypes\0e\00\00\00\02\00\00\00\00\00s\00\00\00\00\00\00\00\00\8c\01\0b.debug_line|\00\00\00\04\00?\00\00\00\01\01\01\fb\0e\0d\00\01\01\01\01\00\00\00\01\00\00\01library/core/src\00\00panicking.rs\00\01\00\00any.rs\00\01\00\00\00\00\05\02K\00\00\00\033\01\05\0e\0a\03\14\08<\06\03\b8\7f\02C\01\03\c8\00J\02\08\00\01\01\04\02\00\05\02\af\00\00\00\03\87\01\01\05\06\0a\ca\02\14\00\01\01\00\91\02\04name\01\e9\01\05\007fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2\016fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5\02\04main\03-core::panicking::panic_fmt::h78607b33a29a727d\04@_$LT$T$u20$as$u20$core..any..Any$GT$::type_id::hed637ffe26dba6a3\07\12\01\00\0f__stack_pointer\09\0a\01\00\07.rodata\00U\09producers\02\08language\01\04Rust\00\0cprocessed-by\01\05rustc%1.75.0-nightly (4b85902b4 2023-11-04)\009\0ftarget_features\03+\0bbulk-memory+\0fmutable-globals+\08sign-extexamples/src/lib.rs\00 \07\01\00\13\00\00\00D\00\00\00#\00\00\0000010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899 out of range for slice of length range end index \00\00.\08\01\00\10\00\00\00\0c\08\01\00\22\00\00\00"))
