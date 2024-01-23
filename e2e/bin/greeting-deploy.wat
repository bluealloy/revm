(module
  (type (;0;) (func (param i32 i32)))
  (type (;1;) (func (param i32)))
  (type (;2;) (func (param i32 i32 i32) (result i32)))
  (type (;3;) (func))
  (import "fluentbase_v1alpha" "_sys_write" (func $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2 (type 0)))
  (import "fluentbase_v1alpha" "_sys_halt" (func $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5 (type 1)))
  (func $compiler_builtins::mem::memcpy::h4ab6845275ba77f4 (type 2) (param i32 i32 i32) (result i32)
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
  (func $memcpy (type 2) (param i32 i32 i32) (result i32)
    local.get 0
    local.get 1
    local.get 2
    call $compiler_builtins::mem::memcpy::h4ab6845275ba77f4)
  (func $deploy (type 3)
    (local i32)
    global.get $__stack_pointer
    i32.const 560
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 12
    i32.add
    i32.const 65536
    i32.const 547
    call $memcpy
    drop
    local.get 0
    i64.const 2349347110920
    i64.store offset=4 align=4
    local.get 0
    i32.const 4
    i32.add
    i32.const 555
    call $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2
    i32.const 0
    call $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5
    unreachable)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 65536))
  (export "memory" (memory 0))
  (export "deploy" (func $deploy))
  (data $.rodata (i32.const 65536) "\00asm\01\00\00\00\01\0d\03`\02\7f\7f\00`\01\7f\00`\00\00\02@\02\12fluentbase_v1alpha\0a_sys_write\00\00\12fluentbase_v1alpha\09_sys_halt\00\01\03\02\01\02\05\03\01\00\02\06\08\01\7f\01A\80\80\04\0b\07\11\02\06memory\02\00\04main\00\02\0aX\01V\01\01\7f#\80\80\80\80\00A k\22\00$\80\80\80\80\00 \00A\1cjA\00(\00\88\80\84\80\006\02\00 \00B\88\80\80\80\c0\017\02\0c \00A\00)\00\80\80\84\80\007\02\14 \00A\0cjA\14\10\80\80\80\80\00A\00\10\81\80\80\80\00\00\0b\0b\14\01\00A\80\80\04\0b\0cHello, World\00\9f\01\04name\01x\03\007fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2\016fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5\02\04main\07\12\01\00\0f__stack_pointer\09\0a\01\00\07.rodata\00U\09producers\02\08language\01\04Rust\00\0cprocessed-by\01\05rustc%1.75.0-nightly (4b85902b4 2023-11-04)\009\0ftarget_features\03+\0bbulk-memory+\0fmutable-globals+\08sign-ext"))
