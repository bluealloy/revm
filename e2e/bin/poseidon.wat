(module
  (type (;0;) (func (param i32 i32 i32)))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (type (;2;) (func (param i32 i32) (result i32)))
  (type (;3;) (func (param i32 i32)))
  (type (;4;) (func (param i32)))
  (type (;5;) (func))
  (type (;6;) (func (param i32 i32 i32 i32)))
  (type (;7;) (func (param i32 i32 i32 i32 i32 i32)))
  (type (;8;) (func (param i32 i32 i32 i32) (result i32)))
  (import "fluentbase_v1alpha" "_sys_write" (func $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2 (type 3)))
  (import "fluentbase_v1alpha" "_sys_halt" (func $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5 (type 4)))
  (import "fluentbase_v1alpha" "_crypto_poseidon" (func $fluentbase_sdk::bindings::_crypto_poseidon::h9c370ab77dc661da (type 0)))
  (import "fluentbase_v1alpha" "_sys_read" (func $fluentbase_sdk::bindings::_sys_read::he28ad825855b6201 (type 0)))
  (func $fluentbase_codec::encoder::Encoder::encode_to_vec::h833e74848d11891c (type 3) (param i32 i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    i32.const 0
    i32.load8_u offset=66436
    drop
    block  ;; label = @1
      block  ;; label = @2
        i32.const 0
        i32.load offset=66440
        local.tee 3
        i32.const 8
        i32.add
        local.tee 4
        i32.const 0
        i32.load offset=66444
        i32.le_u
        br_if 0 (;@2;)
        i32.const 1
        memory.grow
        local.tee 3
        i32.const -1
        i32.eq
        br_if 1 (;@1;)
        i32.const 0
        i32.load offset=66444
        local.set 4
        i32.const 0
        local.get 3
        i32.const 16
        i32.shl
        local.tee 3
        i32.const 65536
        i32.add
        i32.store offset=66444
        i32.const 0
        i32.load offset=66440
        local.get 3
        local.get 3
        local.get 4
        i32.eq
        select
        local.tee 3
        i32.const 8
        i32.add
        local.set 4
      end
      i32.const 0
      local.get 4
      i32.store offset=66440
      local.get 3
      i32.eqz
      br_if 0 (;@1;)
      local.get 3
      i64.const 0
      i64.store align=1
      local.get 1
      i32.const 4
      i32.add
      i32.load
      local.set 5
      local.get 3
      local.get 1
      i32.const 8
      i32.add
      i32.load
      local.tee 1
      i32.store offset=4 align=1
      local.get 3
      i32.const 8
      i32.store align=1
      local.get 2
      i64.const 34359738376
      i64.store offset=8 align=4
      local.get 2
      local.get 3
      i32.store offset=4
      i32.const 8
      local.set 4
      block  ;; label = @2
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        i32.const 4
        i32.add
        i32.const 8
        local.get 1
        call $alloc::raw_vec::RawVec$LT$T$C$A$GT$::reserve::do_reserve_and_handle::hb23085aeff9b1454
        local.get 2
        i32.load offset=4
        local.set 3
        local.get 2
        i32.load offset=12
        local.set 4
      end
      local.get 3
      local.get 4
      i32.add
      local.get 5
      local.get 1
      memory.copy
      local.get 0
      i32.const 8
      i32.add
      local.get 4
      local.get 1
      i32.add
      i32.store
      local.get 0
      local.get 2
      i64.load offset=4 align=4
      i64.store align=4
      local.get 2
      i32.const 16
      i32.add
      global.set $__stack_pointer
      return
    end
    i32.const 8
    call $__rust_alloc_error_handler
    unreachable)
  (func $alloc::raw_vec::RawVec$LT$T$C$A$GT$::reserve::do_reserve_and_handle::hb23085aeff9b1454 (type 0) (param i32 i32 i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        local.get 2
        i32.add
        local.tee 2
        local.get 1
        i32.lt_u
        br_if 0 (;@2;)
        local.get 0
        i32.load offset=4
        local.tee 1
        i32.const 1
        i32.shl
        local.tee 4
        local.get 2
        local.get 4
        local.get 2
        i32.gt_u
        select
        local.tee 2
        i32.const 8
        local.get 2
        i32.const 8
        i32.gt_u
        select
        local.tee 2
        i32.const -1
        i32.xor
        i32.const 31
        i32.shr_u
        local.set 4
        block  ;; label = @3
          block  ;; label = @4
            local.get 1
            br_if 0 (;@4;)
            local.get 3
            i32.const 0
            i32.store offset=24
            br 1 (;@3;)
          end
          local.get 3
          local.get 1
          i32.store offset=28
          local.get 3
          i32.const 1
          i32.store offset=24
          local.get 3
          local.get 0
          i32.load
          i32.store offset=20
        end
        local.get 3
        i32.const 8
        i32.add
        local.get 4
        local.get 2
        local.get 3
        i32.const 20
        i32.add
        call $alloc::raw_vec::finish_grow::hfa2e370a38b88d1e
        local.get 3
        i32.load offset=12
        local.set 1
        block  ;; label = @3
          local.get 3
          i32.load offset=8
          br_if 0 (;@3;)
          local.get 0
          local.get 2
          i32.store offset=4
          local.get 0
          local.get 1
          i32.store
          br 2 (;@1;)
        end
        local.get 1
        i32.const -2147483647
        i32.eq
        br_if 1 (;@1;)
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 3
        i32.const 16
        i32.add
        i32.load
        call $__rust_alloc_error_handler
        unreachable
      end
      call $alloc::raw_vec::capacity_overflow::h26cdf55d7b744af0
      unreachable
    end
    local.get 3
    i32.const 32
    i32.add
    global.set $__stack_pointer)
  (func $__rust_alloc_error_handler (type 4) (param i32)
    local.get 0
    call $__rdl_oom
    unreachable)
  (func $_$LT$T$u20$as$u20$core..convert..Into$LT$U$GT$$GT$::into::h70629df368ddfe16 (type 3) (param i32 i32)
    (local i32 i32)
    i32.const 0
    i32.load8_u offset=66436
    drop
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          i32.const 0
          i32.load offset=66440
          local.tee 2
          i32.const 32
          i32.add
          local.tee 3
          i32.const 0
          i32.load offset=66444
          i32.le_u
          br_if 0 (;@3;)
          i32.const 1
          memory.grow
          local.tee 2
          i32.const -1
          i32.eq
          br_if 1 (;@2;)
          i32.const 0
          i32.load offset=66444
          local.set 3
          i32.const 0
          local.get 2
          i32.const 16
          i32.shl
          local.tee 2
          i32.const 65536
          i32.add
          i32.store offset=66444
          i32.const 0
          i32.load offset=66440
          local.get 2
          local.get 2
          local.get 3
          i32.eq
          select
          local.tee 2
          i32.const 32
          i32.add
          local.set 3
        end
        i32.const 0
        local.get 3
        i32.store offset=66440
        local.get 2
        br_if 1 (;@1;)
      end
      i32.const 32
      call $__rust_alloc_error_handler
      unreachable
    end
    local.get 0
    i32.const 32
    i32.store offset=8
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 2
    local.get 1
    i64.load align=1
    i64.store align=1
    local.get 2
    i32.const 24
    i32.add
    local.get 1
    i32.const 24
    i32.add
    i64.load align=1
    i64.store align=1
    local.get 2
    i32.const 16
    i32.add
    local.get 1
    i32.const 16
    i32.add
    i64.load align=1
    i64.store align=1
    local.get 2
    i32.const 8
    i32.add
    local.get 1
    i32.const 8
    i32.add
    i64.load align=1
    i64.store align=1
    local.get 0
    local.get 2
    local.get 2
    i32.const 1
    i32.or
    local.get 2
    i32.const 1
    i32.and
    local.tee 1
    select
    i32.store offset=12
    local.get 0
    i32.const 65776
    i32.const 65764
    local.get 1
    select
    i32.store)
  (func $main (type 5)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 112
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    call $fluentbase_sdk::evm::ExecutionContext::contract_input::h3893abe9ab2a4288
    local.get 0
    i32.const 16
    i32.add
    i32.const 24
    i32.add
    local.tee 1
    i64.const 0
    i64.store
    local.get 0
    i32.const 16
    i32.add
    i32.const 16
    i32.add
    local.tee 2
    i64.const 0
    i64.store
    local.get 0
    i32.const 16
    i32.add
    i32.const 8
    i32.add
    local.tee 3
    i64.const 0
    i64.store
    local.get 0
    i64.const 0
    i64.store offset=16
    local.get 0
    i32.load offset=4
    local.get 0
    i32.load offset=8
    local.get 0
    i32.const 16
    i32.add
    call $fluentbase_sdk::bindings::_crypto_poseidon::h9c370ab77dc661da
    local.get 0
    i32.const 48
    i32.add
    i32.const 24
    i32.add
    local.get 1
    i64.load
    i64.store
    local.get 0
    i32.const 48
    i32.add
    i32.const 16
    i32.add
    local.get 2
    i64.load
    i64.store
    local.get 0
    i32.const 48
    i32.add
    i32.const 8
    i32.add
    local.get 3
    i64.load
    i64.store
    local.get 0
    local.get 0
    i64.load offset=16
    i64.store offset=48
    local.get 0
    i32.const 84
    i32.add
    local.get 0
    i32.const 48
    i32.add
    call $_$LT$T$u20$as$u20$core..convert..Into$LT$U$GT$$GT$::into::h70629df368ddfe16
    local.get 0
    i32.const 100
    i32.add
    local.get 0
    i32.const 84
    i32.add
    call $fluentbase_codec::encoder::Encoder::encode_to_vec::h833e74848d11891c
    local.get 0
    i32.load offset=100
    local.get 0
    i32.load offset=108
    call $fluentbase_sdk::bindings::_sys_write::h4178963c4d0cfeb2
    i32.const 0
    call $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5
    unreachable)
  (func $fluentbase_sdk::evm::ExecutionContext::contract_input::h3893abe9ab2a4288 (type 4) (param i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    i64.const 0
    i64.store offset=8
    local.get 1
    i32.const 8
    i32.add
    i32.const 92
    i32.const 8
    call $fluentbase_sdk::bindings::_sys_read::he28ad825855b6201
    local.get 1
    i64.const 0
    i64.store offset=24 align=4
    local.get 1
    i32.const 66392
    i32.store offset=20
    local.get 1
    i32.const 66392
    i32.store offset=16
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      local.get 1
                      i32.load offset=12
                      local.tee 2
                      i32.eqz
                      br_if 0 (;@9;)
                      local.get 1
                      i32.load offset=8
                      local.tee 3
                      local.get 2
                      i32.add
                      local.tee 4
                      i32.eqz
                      br_if 1 (;@8;)
                      local.get 4
                      i32.const -1
                      i32.le_s
                      br_if 2 (;@7;)
                      block  ;; label = @10
                        i32.const 0
                        i32.load offset=66440
                        local.tee 5
                        local.get 4
                        i32.add
                        local.tee 6
                        i32.const 0
                        i32.load offset=66444
                        i32.le_u
                        br_if 0 (;@10;)
                        local.get 4
                        i32.const 65535
                        i32.add
                        local.tee 6
                        i32.const 16
                        i32.shr_u
                        memory.grow
                        local.tee 5
                        i32.const -1
                        i32.eq
                        br_if 4 (;@6;)
                        i32.const 0
                        i32.load offset=66444
                        local.set 7
                        i32.const 0
                        local.get 5
                        i32.const 16
                        i32.shl
                        local.tee 5
                        local.get 6
                        i32.const -65536
                        i32.and
                        i32.add
                        i32.store offset=66444
                        i32.const 0
                        i32.load offset=66440
                        local.get 5
                        local.get 5
                        local.get 7
                        i32.eq
                        select
                        local.tee 5
                        local.get 4
                        i32.add
                        local.set 6
                      end
                      i32.const 0
                      local.get 6
                      i32.store offset=66440
                      local.get 5
                      i32.eqz
                      br_if 3 (;@6;)
                      local.get 5
                      i32.const 0
                      local.get 4
                      memory.fill
                      local.get 4
                      i32.const 7
                      i32.le_u
                      br_if 4 (;@5;)
                      local.get 5
                      local.get 1
                      i64.load offset=8
                      i64.store align=1
                      local.get 3
                      local.get 4
                      i32.gt_u
                      br_if 5 (;@4;)
                      local.get 5
                      local.get 3
                      i32.add
                      local.get 3
                      local.get 2
                      call $fluentbase_sdk::bindings::_sys_read::he28ad825855b6201
                      local.get 5
                      i32.load offset=4 align=1
                      local.tee 3
                      local.get 5
                      i32.load align=1
                      local.tee 6
                      i32.add
                      local.tee 2
                      local.get 3
                      i32.lt_u
                      br_if 6 (;@3;)
                      local.get 2
                      local.get 4
                      i32.gt_u
                      br_if 7 (;@2;)
                      block  ;; label = @10
                        block  ;; label = @11
                          local.get 3
                          br_if 0 (;@11;)
                          i32.const 65752
                          local.set 5
                          i32.const 0
                          local.set 2
                          i32.const 66392
                          local.set 4
                          br 1 (;@10;)
                        end
                        local.get 3
                        i32.const -1
                        i32.le_s
                        br_if 3 (;@7;)
                        i32.const 0
                        i32.load8_u offset=66436
                        drop
                        block  ;; label = @11
                          i32.const 0
                          i32.load offset=66440
                          local.tee 4
                          local.get 3
                          i32.add
                          local.tee 2
                          i32.const 0
                          i32.load offset=66444
                          i32.le_u
                          br_if 0 (;@11;)
                          local.get 3
                          i32.const 65535
                          i32.add
                          local.tee 2
                          i32.const 16
                          i32.shr_u
                          memory.grow
                          local.tee 4
                          i32.const -1
                          i32.eq
                          br_if 10 (;@1;)
                          i32.const 0
                          i32.load offset=66444
                          local.set 7
                          i32.const 0
                          local.get 4
                          i32.const 16
                          i32.shl
                          local.tee 4
                          local.get 2
                          i32.const -65536
                          i32.and
                          i32.add
                          i32.store offset=66444
                          i32.const 0
                          i32.load offset=66440
                          local.get 4
                          local.get 4
                          local.get 7
                          i32.eq
                          select
                          local.tee 4
                          local.get 3
                          i32.add
                          local.set 2
                        end
                        i32.const 0
                        local.get 2
                        i32.store offset=66440
                        local.get 4
                        i32.eqz
                        br_if 9 (;@1;)
                        local.get 4
                        local.get 5
                        local.get 6
                        i32.add
                        local.get 3
                        memory.copy
                        block  ;; label = @11
                          local.get 4
                          i32.const 1
                          i32.and
                          i32.eqz
                          br_if 0 (;@11;)
                          i32.const 65776
                          local.set 5
                          local.get 4
                          local.set 2
                          br 1 (;@10;)
                        end
                        local.get 4
                        i32.const 1
                        i32.or
                        local.set 2
                        i32.const 65764
                        local.set 5
                      end
                      local.get 1
                      i32.const 28
                      i32.add
                      local.get 1
                      i32.load offset=20
                      local.get 1
                      i32.load offset=24
                      local.get 1
                      i32.load offset=16
                      i32.load offset=8
                      call_indirect (type 0)
                      local.get 1
                      local.get 2
                      i32.store offset=28
                      local.get 1
                      local.get 3
                      i32.store offset=24
                      local.get 1
                      local.get 4
                      i32.store offset=20
                      local.get 1
                      local.get 5
                      i32.store offset=16
                    end
                    local.get 0
                    local.get 1
                    i64.load offset=16 align=4
                    i64.store align=4
                    local.get 0
                    i32.const 8
                    i32.add
                    local.get 1
                    i32.const 16
                    i32.add
                    i32.const 8
                    i32.add
                    i64.load align=4
                    i64.store align=4
                    local.get 1
                    i32.const 32
                    i32.add
                    global.set $__stack_pointer
                    return
                  end
                  i32.const 8
                  i32.const 0
                  i32.const 66420
                  call $core::slice::index::slice_end_index_len_fail::h6372e465cf26b33a
                  unreachable
                end
                call $alloc::raw_vec::capacity_overflow::h26cdf55d7b744af0
                unreachable
              end
              local.get 4
              call $__rust_alloc_error_handler
              unreachable
            end
            i32.const 8
            local.get 4
            i32.const 66420
            call $core::slice::index::slice_end_index_len_fail::h6372e465cf26b33a
            unreachable
          end
          local.get 3
          local.get 4
          i32.const 66420
          call $core::slice::index::slice_index_order_fail::h5c05174755728e22
          unreachable
        end
        local.get 6
        local.get 2
        i32.const 66376
        call $core::slice::index::slice_index_order_fail::h5c05174755728e22
        unreachable
      end
      local.get 2
      local.get 4
      i32.const 66376
      call $core::slice::index::slice_end_index_len_fail::h6372e465cf26b33a
      unreachable
    end
    local.get 3
    call $__rust_alloc_error_handler
    unreachable)
  (func $__rdl_oom (type 4) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    local.get 0
    i32.store offset=12
    local.get 1
    i32.const 28
    i32.add
    i64.const 1
    i64.store align=4
    local.get 1
    i32.const 2
    i32.store offset=20
    local.get 1
    i32.const 65644
    i32.store offset=16
    local.get 1
    i32.const 1
    i32.store offset=44
    local.get 1
    local.get 1
    i32.const 40
    i32.add
    i32.store offset=24
    local.get 1
    local.get 1
    i32.const 12
    i32.add
    i32.store offset=40
    local.get 1
    i32.const 16
    i32.add
    call $core::panicking::panic_nounwind_fmt::hc0791cac263d58db
    unreachable)
  (func $alloc::raw_vec::capacity_overflow::h26cdf55d7b744af0 (type 5)
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
    i32.const 65584
    i32.store offset=8
    local.get 0
    i32.const 66392
    i32.store offset=16
    local.get 0
    i32.const 8
    i32.add
    i32.const 65592
    call $core::panicking::panic_fmt::h78607b33a29a727d
    unreachable)
  (func $core::panicking::panic_fmt::h78607b33a29a727d (type 3) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 66392
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
  (func $core::fmt::num::imp::_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$::fmt::he696c0e431156bce (type 2) (param i32 i32) (result i32)
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
        i32.const 66024
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
        i32.const 66024
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
      i32.const 66024
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
        i32.const 66024
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
    i32.const 66392
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
        call_indirect (type 1)
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
        call_indirect (type 1)
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
            call_indirect (type 2)
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
        call_indirect (type 1)
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
          call_indirect (type 2)
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
      call_indirect (type 1)
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
        call_indirect (type 2)
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
  (func $core::panicking::panic_nounwind_fmt::hc0791cac263d58db (type 4) (param i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 1
    global.set $__stack_pointer
    local.get 1
    i32.const 66392
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
  (func $bytes::bytes::promotable_even_clone::h625046199c91628d (type 6) (param i32 i32 i32 i32)
    (local i32)
    block  ;; label = @1
      local.get 1
      i32.load
      local.tee 4
      i32.const 1
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 1
      local.get 4
      local.get 4
      i32.const -2
      i32.and
      local.get 2
      local.get 3
      call $bytes::bytes::shallow_clone_vec::ha03fb4f0257441cd
      return
    end
    local.get 4
    local.get 4
    i32.load offset=8
    local.tee 1
    i32.const 1
    i32.add
    i32.store offset=8
    block  ;; label = @1
      local.get 1
      i32.const -1
      i32.le_s
      br_if 0 (;@1;)
      local.get 0
      local.get 4
      i32.store offset=12
      local.get 0
      local.get 3
      i32.store offset=8
      local.get 0
      local.get 2
      i32.store offset=4
      local.get 0
      i32.const 65880
      i32.store
      return
    end
    call $bytes::abort::h93cd02066bd444b8
    unreachable)
  (func $bytes::bytes::shallow_clone_vec::ha03fb4f0257441cd (type 7) (param i32 i32 i32 i32 i32 i32)
    (local i32 i32)
    i32.const 0
    i32.load8_u offset=66436
    drop
    block  ;; label = @1
      i32.const 0
      i32.load offset=66440
      local.tee 6
      i32.const 3
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      i32.const 0
      local.get 6
      i32.const -4
      i32.and
      i32.const 4
      i32.add
      local.tee 6
      i32.store offset=66440
    end
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 6
          i32.const 12
          i32.add
          local.tee 7
          i32.const 0
          i32.load offset=66444
          i32.le_u
          br_if 0 (;@3;)
          i32.const 1
          memory.grow
          local.tee 6
          i32.const -1
          i32.eq
          br_if 1 (;@2;)
          i32.const 0
          i32.load offset=66444
          local.set 7
          i32.const 0
          local.get 6
          i32.const 16
          i32.shl
          local.tee 6
          i32.const 65536
          i32.add
          i32.store offset=66444
          i32.const 0
          i32.load offset=66440
          local.get 6
          local.get 6
          local.get 7
          i32.eq
          select
          local.tee 6
          i32.const 12
          i32.add
          local.set 7
        end
        i32.const 0
        local.get 7
        i32.store offset=66440
        local.get 6
        i32.eqz
        br_if 0 (;@2;)
        local.get 6
        i32.const 2
        i32.store offset=8
        local.get 6
        local.get 3
        i32.store
        local.get 6
        local.get 4
        local.get 3
        i32.sub
        local.get 5
        i32.add
        i32.store offset=4
        local.get 1
        local.get 6
        local.get 1
        i32.load
        local.tee 3
        local.get 3
        local.get 2
        i32.eq
        local.tee 7
        select
        i32.store
        block  ;; label = @3
          local.get 7
          br_if 0 (;@3;)
          local.get 3
          local.get 3
          i32.load offset=8
          local.tee 1
          i32.const 1
          i32.add
          i32.store offset=8
          local.get 3
          local.set 6
          local.get 1
          i32.const -1
          i32.le_s
          br_if 2 (;@1;)
        end
        local.get 0
        local.get 6
        i32.store offset=12
        local.get 0
        local.get 5
        i32.store offset=8
        local.get 0
        local.get 4
        i32.store offset=4
        local.get 0
        i32.const 65880
        i32.store
        return
      end
      i32.const 12
      call $__rust_alloc_error_handler
      unreachable
    end
    call $bytes::abort::h93cd02066bd444b8
    unreachable)
  (func $bytes::abort::h93cd02066bd444b8 (type 5)
    call $core::panicking::panic::hdd77bb12897b1389
    unreachable)
  (func $bytes::bytes::promotable_even_to_vec::h09f3a4302a0af998 (type 6) (param i32 i32 i32 i32)
    block  ;; label = @1
      local.get 1
      i32.load
      local.tee 1
      i32.const 1
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      i32.const -2
      i32.and
      local.tee 1
      local.get 2
      local.get 3
      memory.copy
      local.get 0
      local.get 3
      i32.store offset=8
      local.get 0
      local.get 2
      local.get 3
      i32.add
      local.get 1
      i32.sub
      i32.store offset=4
      local.get 0
      local.get 1
      i32.store
      return
    end
    local.get 0
    local.get 1
    local.get 2
    local.get 3
    call $bytes::bytes::shared_to_vec_impl::h0482b16c878678eb)
  (func $bytes::bytes::shared_to_vec_impl::h0482b16c878678eb (type 6) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 4
    global.set $__stack_pointer
    i32.const 1
    local.set 5
    local.get 1
    i32.const 0
    local.get 1
    i32.load offset=8
    local.tee 6
    local.get 6
    i32.const 1
    i32.eq
    select
    i32.store offset=8
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 6
              i32.const 1
              i32.ne
              br_if 0 (;@5;)
              local.get 1
              i32.load offset=4
              local.set 6
              local.get 1
              i32.load
              local.tee 1
              local.get 2
              local.get 3
              memory.copy
              local.get 0
              local.get 6
              i32.store offset=4
              local.get 0
              local.get 1
              i32.store
              br 1 (;@4;)
            end
            block  ;; label = @5
              local.get 3
              i32.eqz
              br_if 0 (;@5;)
              local.get 3
              i32.const -1
              i32.le_s
              br_if 2 (;@3;)
              i32.const 0
              i32.load8_u offset=66436
              drop
              block  ;; label = @6
                i32.const 0
                i32.load offset=66440
                local.tee 5
                local.get 3
                i32.add
                local.tee 6
                i32.const 0
                i32.load offset=66444
                i32.le_u
                br_if 0 (;@6;)
                local.get 3
                i32.const 65535
                i32.add
                local.tee 5
                i32.const 16
                i32.shr_u
                memory.grow
                local.tee 6
                i32.const -1
                i32.eq
                br_if 4 (;@2;)
                i32.const 0
                i32.load offset=66444
                local.set 7
                i32.const 0
                local.get 6
                i32.const 16
                i32.shl
                local.tee 6
                local.get 5
                i32.const -65536
                i32.and
                i32.add
                i32.store offset=66444
                i32.const 0
                i32.load offset=66440
                local.get 6
                local.get 6
                local.get 7
                i32.eq
                select
                local.tee 5
                local.get 3
                i32.add
                local.set 6
              end
              i32.const 0
              local.get 6
              i32.store offset=66440
              local.get 5
              i32.eqz
              br_if 3 (;@2;)
            end
            local.get 5
            local.get 2
            local.get 3
            memory.copy
            local.get 1
            local.get 1
            i32.load offset=8
            local.tee 6
            i32.const -1
            i32.add
            i32.store offset=8
            block  ;; label = @5
              local.get 6
              i32.const 1
              i32.ne
              br_if 0 (;@5;)
              local.get 1
              i32.const 4
              i32.add
              i32.load
              i32.const -1
              i32.le_s
              br_if 4 (;@1;)
            end
            local.get 0
            local.get 3
            i32.store offset=4
            local.get 0
            local.get 5
            i32.store
          end
          local.get 0
          local.get 3
          i32.store offset=8
          local.get 4
          i32.const 16
          i32.add
          global.set $__stack_pointer
          return
        end
        call $alloc::raw_vec::capacity_overflow::h26cdf55d7b744af0
        unreachable
      end
      local.get 3
      call $__rust_alloc_error_handler
      unreachable
    end
    local.get 4
    i32.const 15
    i32.add
    i32.const 65864
    call $core::result::unwrap_failed::h38a5f72e87633ead
    unreachable)
  (func $bytes::bytes::promotable_even_drop::hb0bc4b7b98503867 (type 0) (param i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.load
          local.tee 0
          i32.const 1
          i32.and
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          local.get 0
          i32.const -2
          i32.and
          i32.sub
          local.get 2
          i32.add
          i32.const -1
          i32.gt_s
          br_if 1 (;@2;)
          local.get 3
          i32.const 15
          i32.add
          i32.const 65848
          call $core::result::unwrap_failed::h38a5f72e87633ead
          unreachable
        end
        local.get 0
        local.get 0
        i32.load offset=8
        local.tee 2
        i32.const -1
        i32.add
        i32.store offset=8
        local.get 2
        i32.const 1
        i32.ne
        br_if 0 (;@2;)
        local.get 0
        i32.const 4
        i32.add
        i32.load
        i32.const -1
        i32.le_s
        br_if 1 (;@1;)
      end
      local.get 3
      i32.const 16
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 3
    i32.const 15
    i32.add
    i32.const 65864
    call $core::result::unwrap_failed::h38a5f72e87633ead
    unreachable)
  (func $core::result::unwrap_failed::h38a5f72e87633ead (type 3) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 64
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 43
    i32.store offset=12
    local.get 2
    i32.const 65788
    i32.store offset=8
    local.get 2
    i32.const 65832
    i32.store offset=20
    local.get 2
    local.get 0
    i32.store offset=16
    local.get 2
    i32.const 24
    i32.add
    i32.const 12
    i32.add
    i64.const 2
    i64.store align=4
    local.get 2
    i32.const 48
    i32.add
    i32.const 12
    i32.add
    i32.const 2
    i32.store
    local.get 2
    i32.const 2
    i32.store offset=28
    local.get 2
    i32.const 66008
    i32.store offset=24
    local.get 2
    i32.const 3
    i32.store offset=52
    local.get 2
    local.get 2
    i32.const 48
    i32.add
    i32.store offset=32
    local.get 2
    local.get 2
    i32.const 16
    i32.add
    i32.store offset=56
    local.get 2
    local.get 2
    i32.const 8
    i32.add
    i32.store offset=48
    local.get 2
    i32.const 24
    i32.add
    local.get 1
    call $core::panicking::panic_fmt::h78607b33a29a727d
    unreachable)
  (func $core::ptr::drop_in_place$LT$core..alloc..layout..LayoutError$GT$::hc14ab9179ef63929 (type 4) (param i32))
  (func $core::panicking::panic::hdd77bb12897b1389 (type 5)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 0
    global.set $__stack_pointer
    local.get 0
    i32.const 12
    i32.add
    i64.const 0
    i64.store align=4
    local.get 0
    i32.const 1
    i32.store offset=4
    local.get 0
    i32.const 66392
    i32.store offset=8
    local.get 0
    i32.const 5
    i32.store offset=28
    local.get 0
    i32.const 65892
    i32.store offset=24
    local.get 0
    local.get 0
    i32.const 24
    i32.add
    i32.store
    local.get 0
    i32.const 65988
    call $core::panicking::panic_fmt::h78607b33a29a727d
    unreachable)
  (func $bytes::bytes::shared_clone::h19b57f066b9013d5 (type 6) (param i32 i32 i32 i32)
    (local i32)
    local.get 1
    i32.load
    local.tee 1
    local.get 1
    i32.load offset=8
    local.tee 4
    i32.const 1
    i32.add
    i32.store offset=8
    block  ;; label = @1
      local.get 4
      i32.const -1
      i32.gt_s
      br_if 0 (;@1;)
      call $bytes::abort::h93cd02066bd444b8
      unreachable
    end
    local.get 0
    local.get 1
    i32.store offset=12
    local.get 0
    local.get 3
    i32.store offset=8
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 0
    i32.const 65880
    i32.store)
  (func $bytes::bytes::shared_to_vec::h0a07e611a75997a4 (type 6) (param i32 i32 i32 i32)
    local.get 0
    local.get 1
    i32.load
    local.get 2
    local.get 3
    call $bytes::bytes::shared_to_vec_impl::h0482b16c878678eb)
  (func $bytes::bytes::shared_drop::h78acd3e84919f2f9 (type 0) (param i32 i32 i32)
    (local i32 i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 0
    i32.load
    local.tee 0
    local.get 0
    i32.load offset=8
    local.tee 4
    i32.const -1
    i32.add
    i32.store offset=8
    block  ;; label = @1
      block  ;; label = @2
        local.get 4
        i32.const 1
        i32.ne
        br_if 0 (;@2;)
        local.get 0
        i32.const 4
        i32.add
        i32.load
        i32.const -1
        i32.le_s
        br_if 1 (;@1;)
      end
      local.get 3
      i32.const 16
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 3
    i32.const 15
    i32.add
    i32.const 65864
    call $core::result::unwrap_failed::h38a5f72e87633ead
    unreachable)
  (func $bytes::bytes::promotable_odd_clone::h67309b0b4ba9becd (type 6) (param i32 i32 i32 i32)
    (local i32)
    block  ;; label = @1
      local.get 1
      i32.load
      local.tee 4
      i32.const 1
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 1
      local.get 4
      local.get 4
      local.get 2
      local.get 3
      call $bytes::bytes::shallow_clone_vec::ha03fb4f0257441cd
      return
    end
    local.get 4
    local.get 4
    i32.load offset=8
    local.tee 1
    i32.const 1
    i32.add
    i32.store offset=8
    block  ;; label = @1
      local.get 1
      i32.const -1
      i32.le_s
      br_if 0 (;@1;)
      local.get 0
      local.get 4
      i32.store offset=12
      local.get 0
      local.get 3
      i32.store offset=8
      local.get 0
      local.get 2
      i32.store offset=4
      local.get 0
      i32.const 65880
      i32.store
      return
    end
    call $bytes::abort::h93cd02066bd444b8
    unreachable)
  (func $bytes::bytes::promotable_odd_to_vec::h76d8bdd4b86d6d4f (type 6) (param i32 i32 i32 i32)
    block  ;; label = @1
      local.get 1
      i32.load
      local.tee 1
      i32.const 1
      i32.and
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      local.get 2
      local.get 3
      memory.copy
      local.get 0
      local.get 3
      i32.store offset=8
      local.get 0
      local.get 1
      i32.store
      local.get 0
      local.get 2
      local.get 3
      i32.add
      local.get 1
      i32.sub
      i32.store offset=4
      return
    end
    local.get 0
    local.get 1
    local.get 2
    local.get 3
    call $bytes::bytes::shared_to_vec_impl::h0482b16c878678eb)
  (func $bytes::bytes::promotable_odd_drop::h237c4ebddc6329dc (type 0) (param i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 0
          i32.load
          local.tee 0
          i32.const 1
          i32.and
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          local.get 0
          i32.sub
          local.get 2
          i32.add
          i32.const -1
          i32.gt_s
          br_if 1 (;@2;)
          local.get 3
          i32.const 15
          i32.add
          i32.const 65848
          call $core::result::unwrap_failed::h38a5f72e87633ead
          unreachable
        end
        local.get 0
        local.get 0
        i32.load offset=8
        local.tee 2
        i32.const -1
        i32.add
        i32.store offset=8
        local.get 2
        i32.const 1
        i32.ne
        br_if 0 (;@2;)
        local.get 0
        i32.const 4
        i32.add
        i32.load
        i32.const -1
        i32.le_s
        br_if 1 (;@1;)
      end
      local.get 3
      i32.const 16
      i32.add
      global.set $__stack_pointer
      return
    end
    local.get 3
    i32.const 15
    i32.add
    i32.const 65864
    call $core::result::unwrap_failed::h38a5f72e87633ead
    unreachable)
  (func $bytes::bytes::static_clone::hfe2f8292696dda85 (type 6) (param i32 i32 i32 i32)
    local.get 0
    i32.const 0
    i32.store offset=12
    local.get 0
    local.get 3
    i32.store offset=8
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 0
    i32.const 65752
    i32.store)
  (func $bytes::bytes::static_to_vec::hb94cb51f4d0e0ea0 (type 6) (param i32 i32 i32 i32)
    (local i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 3
            br_if 0 (;@4;)
            i32.const 1
            local.set 4
            br 1 (;@3;)
          end
          local.get 3
          i32.const -1
          i32.le_s
          br_if 1 (;@2;)
          i32.const 0
          i32.load8_u offset=66436
          drop
          block  ;; label = @4
            i32.const 0
            i32.load offset=66440
            local.tee 4
            local.get 3
            i32.add
            local.tee 5
            i32.const 0
            i32.load offset=66444
            i32.le_u
            br_if 0 (;@4;)
            local.get 3
            i32.const 65535
            i32.add
            local.tee 5
            i32.const 16
            i32.shr_u
            memory.grow
            local.tee 4
            i32.const -1
            i32.eq
            br_if 3 (;@1;)
            i32.const 0
            i32.load offset=66444
            local.set 6
            i32.const 0
            local.get 4
            i32.const 16
            i32.shl
            local.tee 4
            local.get 5
            i32.const -65536
            i32.and
            i32.add
            i32.store offset=66444
            i32.const 0
            i32.load offset=66440
            local.get 4
            local.get 4
            local.get 6
            i32.eq
            select
            local.tee 4
            local.get 3
            i32.add
            local.set 5
          end
          i32.const 0
          local.get 5
          i32.store offset=66440
          local.get 4
          i32.eqz
          br_if 2 (;@1;)
        end
        local.get 4
        local.get 2
        local.get 3
        memory.copy
        local.get 0
        local.get 3
        i32.store offset=8
        local.get 0
        local.get 3
        i32.store offset=4
        local.get 0
        local.get 4
        i32.store
        return
      end
      call $alloc::raw_vec::capacity_overflow::h26cdf55d7b744af0
      unreachable
    end
    local.get 3
    call $__rust_alloc_error_handler
    unreachable)
  (func $bytes::bytes::static_drop::h58e729ae4cc2d39d (type 0) (param i32 i32 i32))
  (func $_$LT$T$u20$as$u20$core..any..Any$GT$::type_id::hed637ffe26dba6a3 (type 3) (param i32 i32)
    local.get 0
    i64.const 568815540544143123
    i64.store offset=8
    local.get 0
    i64.const 5657071353825360256
    i64.store)
  (func $core::fmt::Formatter::pad_integral::write_prefix::h43684999422d0638 (type 8) (param i32 i32 i32 i32) (result i32)
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
          call_indirect (type 2)
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
    call_indirect (type 1))
  (func $core::slice::index::slice_end_index_len_fail::h6372e465cf26b33a (type 0) (param i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 0
    i32.store
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    i32.const 8
    i32.add
    i32.const 12
    i32.add
    i64.const 2
    i64.store align=4
    local.get 3
    i32.const 32
    i32.add
    i32.const 12
    i32.add
    i32.const 1
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 66276
    i32.store offset=8
    local.get 3
    i32.const 1
    i32.store offset=36
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    local.get 3
    i32.const 4
    i32.add
    i32.store offset=40
    local.get 3
    local.get 3
    i32.store offset=32
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $core::panicking::panic_fmt::h78607b33a29a727d
    unreachable)
  (func $_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$::fmt::h743d7417ec5e8687 (type 2) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    local.get 0
    i32.load offset=4
    local.set 2
    local.get 0
    i32.load
    local.set 3
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          i32.load
          local.tee 4
          local.get 1
          i32.load offset=8
          local.tee 0
          i32.or
          i32.eqz
          br_if 0 (;@3;)
          block  ;; label = @4
            local.get 0
            i32.eqz
            br_if 0 (;@4;)
            local.get 3
            local.get 2
            i32.add
            local.set 5
            local.get 1
            i32.const 12
            i32.add
            i32.load
            i32.const 1
            i32.add
            local.set 6
            i32.const 0
            local.set 7
            local.get 3
            local.set 8
            block  ;; label = @5
              loop  ;; label = @6
                local.get 8
                local.set 0
                local.get 6
                i32.const -1
                i32.add
                local.tee 6
                i32.eqz
                br_if 1 (;@5;)
                local.get 0
                local.get 5
                i32.eq
                br_if 2 (;@4;)
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 0
                    i32.load8_s
                    local.tee 9
                    i32.const -1
                    i32.le_s
                    br_if 0 (;@8;)
                    local.get 0
                    i32.const 1
                    i32.add
                    local.set 8
                    local.get 9
                    i32.const 255
                    i32.and
                    local.set 9
                    br 1 (;@7;)
                  end
                  local.get 0
                  i32.load8_u offset=1
                  i32.const 63
                  i32.and
                  local.set 10
                  local.get 9
                  i32.const 31
                  i32.and
                  local.set 8
                  block  ;; label = @8
                    local.get 9
                    i32.const -33
                    i32.gt_u
                    br_if 0 (;@8;)
                    local.get 8
                    i32.const 6
                    i32.shl
                    local.get 10
                    i32.or
                    local.set 9
                    local.get 0
                    i32.const 2
                    i32.add
                    local.set 8
                    br 1 (;@7;)
                  end
                  local.get 10
                  i32.const 6
                  i32.shl
                  local.get 0
                  i32.load8_u offset=2
                  i32.const 63
                  i32.and
                  i32.or
                  local.set 10
                  block  ;; label = @8
                    local.get 9
                    i32.const -16
                    i32.ge_u
                    br_if 0 (;@8;)
                    local.get 10
                    local.get 8
                    i32.const 12
                    i32.shl
                    i32.or
                    local.set 9
                    local.get 0
                    i32.const 3
                    i32.add
                    local.set 8
                    br 1 (;@7;)
                  end
                  local.get 10
                  i32.const 6
                  i32.shl
                  local.get 0
                  i32.load8_u offset=3
                  i32.const 63
                  i32.and
                  i32.or
                  local.get 8
                  i32.const 18
                  i32.shl
                  i32.const 1835008
                  i32.and
                  i32.or
                  local.tee 9
                  i32.const 1114112
                  i32.eq
                  br_if 3 (;@4;)
                  local.get 0
                  i32.const 4
                  i32.add
                  local.set 8
                end
                local.get 7
                local.get 0
                i32.sub
                local.get 8
                i32.add
                local.set 7
                local.get 9
                i32.const 1114112
                i32.ne
                br_if 0 (;@6;)
                br 2 (;@4;)
              end
            end
            local.get 0
            local.get 5
            i32.eq
            br_if 0 (;@4;)
            block  ;; label = @5
              local.get 0
              i32.load8_s
              local.tee 8
              i32.const -1
              i32.gt_s
              br_if 0 (;@5;)
              local.get 8
              i32.const -32
              i32.lt_u
              br_if 0 (;@5;)
              local.get 8
              i32.const -16
              i32.lt_u
              br_if 0 (;@5;)
              local.get 0
              i32.load8_u offset=2
              i32.const 63
              i32.and
              i32.const 6
              i32.shl
              local.get 0
              i32.load8_u offset=1
              i32.const 63
              i32.and
              i32.const 12
              i32.shl
              i32.or
              local.get 0
              i32.load8_u offset=3
              i32.const 63
              i32.and
              i32.or
              local.get 8
              i32.const 255
              i32.and
              i32.const 18
              i32.shl
              i32.const 1835008
              i32.and
              i32.or
              i32.const 1114112
              i32.eq
              br_if 1 (;@4;)
            end
            block  ;; label = @5
              block  ;; label = @6
                local.get 7
                i32.eqz
                br_if 0 (;@6;)
                block  ;; label = @7
                  local.get 7
                  local.get 2
                  i32.lt_u
                  br_if 0 (;@7;)
                  i32.const 0
                  local.set 0
                  local.get 7
                  local.get 2
                  i32.eq
                  br_if 1 (;@6;)
                  br 2 (;@5;)
                end
                i32.const 0
                local.set 0
                local.get 3
                local.get 7
                i32.add
                i32.load8_s
                i32.const -64
                i32.lt_s
                br_if 1 (;@5;)
              end
              local.get 3
              local.set 0
            end
            local.get 7
            local.get 2
            local.get 0
            select
            local.set 2
            local.get 0
            local.get 3
            local.get 0
            select
            local.set 3
          end
          block  ;; label = @4
            local.get 4
            br_if 0 (;@4;)
            local.get 1
            i32.load offset=20
            local.get 3
            local.get 2
            local.get 1
            i32.const 24
            i32.add
            i32.load
            i32.load offset=12
            call_indirect (type 1)
            return
          end
          local.get 1
          i32.load offset=4
          local.set 11
          block  ;; label = @4
            local.get 2
            i32.const 16
            i32.lt_u
            br_if 0 (;@4;)
            local.get 2
            local.get 3
            local.get 3
            i32.const 3
            i32.add
            i32.const -4
            i32.and
            local.tee 9
            i32.sub
            local.tee 6
            i32.add
            local.tee 4
            i32.const 3
            i32.and
            local.set 10
            i32.const 0
            local.set 5
            i32.const 0
            local.set 0
            block  ;; label = @5
              local.get 3
              local.get 9
              i32.eq
              br_if 0 (;@5;)
              i32.const 0
              local.set 0
              block  ;; label = @6
                local.get 9
                local.get 3
                i32.const -1
                i32.xor
                i32.add
                i32.const 3
                i32.lt_u
                br_if 0 (;@6;)
                i32.const 0
                local.set 0
                i32.const 0
                local.set 7
                loop  ;; label = @7
                  local.get 0
                  local.get 3
                  local.get 7
                  i32.add
                  local.tee 8
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.get 8
                  i32.const 1
                  i32.add
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.get 8
                  i32.const 2
                  i32.add
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.get 8
                  i32.const 3
                  i32.add
                  i32.load8_s
                  i32.const -65
                  i32.gt_s
                  i32.add
                  local.set 0
                  local.get 7
                  i32.const 4
                  i32.add
                  local.tee 7
                  br_if 0 (;@7;)
                end
              end
              local.get 3
              local.set 8
              loop  ;; label = @6
                local.get 0
                local.get 8
                i32.load8_s
                i32.const -65
                i32.gt_s
                i32.add
                local.set 0
                local.get 8
                i32.const 1
                i32.add
                local.set 8
                local.get 6
                i32.const 1
                i32.add
                local.tee 6
                br_if 0 (;@6;)
              end
            end
            block  ;; label = @5
              local.get 10
              i32.eqz
              br_if 0 (;@5;)
              local.get 9
              local.get 4
              i32.const -4
              i32.and
              i32.add
              local.tee 8
              i32.load8_s
              i32.const -65
              i32.gt_s
              local.set 5
              local.get 10
              i32.const 1
              i32.eq
              br_if 0 (;@5;)
              local.get 5
              local.get 8
              i32.load8_s offset=1
              i32.const -65
              i32.gt_s
              i32.add
              local.set 5
              local.get 10
              i32.const 2
              i32.eq
              br_if 0 (;@5;)
              local.get 5
              local.get 8
              i32.load8_s offset=2
              i32.const -65
              i32.gt_s
              i32.add
              local.set 5
            end
            local.get 4
            i32.const 2
            i32.shr_u
            local.set 7
            local.get 5
            local.get 0
            i32.add
            local.set 10
            loop  ;; label = @5
              local.get 9
              local.set 4
              local.get 7
              i32.eqz
              br_if 4 (;@1;)
              local.get 7
              i32.const 192
              local.get 7
              i32.const 192
              i32.lt_u
              select
              local.tee 5
              i32.const 3
              i32.and
              local.set 12
              local.get 5
              i32.const 2
              i32.shl
              local.set 13
              i32.const 0
              local.set 8
              block  ;; label = @6
                local.get 5
                i32.const 4
                i32.lt_u
                br_if 0 (;@6;)
                local.get 4
                local.get 13
                i32.const 1008
                i32.and
                i32.add
                local.set 6
                i32.const 0
                local.set 8
                local.get 4
                local.set 0
                loop  ;; label = @7
                  local.get 0
                  i32.const 12
                  i32.add
                  i32.load
                  local.tee 9
                  i32.const -1
                  i32.xor
                  i32.const 7
                  i32.shr_u
                  local.get 9
                  i32.const 6
                  i32.shr_u
                  i32.or
                  i32.const 16843009
                  i32.and
                  local.get 0
                  i32.const 8
                  i32.add
                  i32.load
                  local.tee 9
                  i32.const -1
                  i32.xor
                  i32.const 7
                  i32.shr_u
                  local.get 9
                  i32.const 6
                  i32.shr_u
                  i32.or
                  i32.const 16843009
                  i32.and
                  local.get 0
                  i32.const 4
                  i32.add
                  i32.load
                  local.tee 9
                  i32.const -1
                  i32.xor
                  i32.const 7
                  i32.shr_u
                  local.get 9
                  i32.const 6
                  i32.shr_u
                  i32.or
                  i32.const 16843009
                  i32.and
                  local.get 0
                  i32.load
                  local.tee 9
                  i32.const -1
                  i32.xor
                  i32.const 7
                  i32.shr_u
                  local.get 9
                  i32.const 6
                  i32.shr_u
                  i32.or
                  i32.const 16843009
                  i32.and
                  local.get 8
                  i32.add
                  i32.add
                  i32.add
                  i32.add
                  local.set 8
                  local.get 0
                  i32.const 16
                  i32.add
                  local.tee 0
                  local.get 6
                  i32.ne
                  br_if 0 (;@7;)
                end
              end
              local.get 7
              local.get 5
              i32.sub
              local.set 7
              local.get 4
              local.get 13
              i32.add
              local.set 9
              local.get 8
              i32.const 8
              i32.shr_u
              i32.const 16711935
              i32.and
              local.get 8
              i32.const 16711935
              i32.and
              i32.add
              i32.const 65537
              i32.mul
              i32.const 16
              i32.shr_u
              local.get 10
              i32.add
              local.set 10
              local.get 12
              i32.eqz
              br_if 0 (;@5;)
            end
            local.get 4
            local.get 5
            i32.const 252
            i32.and
            i32.const 2
            i32.shl
            i32.add
            local.tee 8
            i32.load
            local.tee 0
            i32.const -1
            i32.xor
            i32.const 7
            i32.shr_u
            local.get 0
            i32.const 6
            i32.shr_u
            i32.or
            i32.const 16843009
            i32.and
            local.set 0
            local.get 12
            i32.const 1
            i32.eq
            br_if 2 (;@2;)
            local.get 8
            i32.load offset=4
            local.tee 9
            i32.const -1
            i32.xor
            i32.const 7
            i32.shr_u
            local.get 9
            i32.const 6
            i32.shr_u
            i32.or
            i32.const 16843009
            i32.and
            local.get 0
            i32.add
            local.set 0
            local.get 12
            i32.const 2
            i32.eq
            br_if 2 (;@2;)
            local.get 8
            i32.load offset=8
            local.tee 8
            i32.const -1
            i32.xor
            i32.const 7
            i32.shr_u
            local.get 8
            i32.const 6
            i32.shr_u
            i32.or
            i32.const 16843009
            i32.and
            local.get 0
            i32.add
            local.set 0
            br 2 (;@2;)
          end
          block  ;; label = @4
            local.get 2
            br_if 0 (;@4;)
            i32.const 0
            local.set 10
            br 3 (;@1;)
          end
          local.get 2
          i32.const 3
          i32.and
          local.set 8
          block  ;; label = @4
            block  ;; label = @5
              local.get 2
              i32.const 4
              i32.ge_u
              br_if 0 (;@5;)
              i32.const 0
              local.set 10
              i32.const 0
              local.set 0
              br 1 (;@4;)
            end
            local.get 3
            i32.load8_s
            i32.const -65
            i32.gt_s
            local.get 3
            i32.load8_s offset=1
            i32.const -65
            i32.gt_s
            i32.add
            local.get 3
            i32.load8_s offset=2
            i32.const -65
            i32.gt_s
            i32.add
            local.get 3
            i32.load8_s offset=3
            i32.const -65
            i32.gt_s
            i32.add
            local.set 10
            local.get 2
            i32.const -4
            i32.and
            local.tee 0
            i32.const 4
            i32.eq
            br_if 0 (;@4;)
            local.get 10
            local.get 3
            i32.load8_s offset=4
            i32.const -65
            i32.gt_s
            i32.add
            local.get 3
            i32.load8_s offset=5
            i32.const -65
            i32.gt_s
            i32.add
            local.get 3
            i32.load8_s offset=6
            i32.const -65
            i32.gt_s
            i32.add
            local.get 3
            i32.load8_s offset=7
            i32.const -65
            i32.gt_s
            i32.add
            local.set 10
            local.get 0
            i32.const 8
            i32.eq
            br_if 0 (;@4;)
            local.get 10
            local.get 3
            i32.load8_s offset=8
            i32.const -65
            i32.gt_s
            i32.add
            local.get 3
            i32.load8_s offset=9
            i32.const -65
            i32.gt_s
            i32.add
            local.get 3
            i32.load8_s offset=10
            i32.const -65
            i32.gt_s
            i32.add
            local.get 3
            i32.load8_s offset=11
            i32.const -65
            i32.gt_s
            i32.add
            local.set 10
          end
          local.get 8
          i32.eqz
          br_if 2 (;@1;)
          local.get 3
          local.get 0
          i32.add
          local.set 0
          loop  ;; label = @4
            local.get 10
            local.get 0
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 10
            local.get 0
            i32.const 1
            i32.add
            local.set 0
            local.get 8
            i32.const -1
            i32.add
            local.tee 8
            br_if 0 (;@4;)
            br 3 (;@1;)
          end
        end
        local.get 1
        i32.load offset=20
        local.get 3
        local.get 2
        local.get 1
        i32.const 24
        i32.add
        i32.load
        i32.load offset=12
        call_indirect (type 1)
        return
      end
      local.get 0
      i32.const 8
      i32.shr_u
      i32.const 459007
      i32.and
      local.get 0
      i32.const 16711935
      i32.and
      i32.add
      i32.const 65537
      i32.mul
      i32.const 16
      i32.shr_u
      local.get 10
      i32.add
      local.set 10
    end
    block  ;; label = @1
      block  ;; label = @2
        local.get 11
        local.get 10
        i32.le_u
        br_if 0 (;@2;)
        local.get 11
        local.get 10
        i32.sub
        local.set 7
        i32.const 0
        local.set 0
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 1
              i32.load8_u offset=32
              br_table 2 (;@3;) 0 (;@5;) 1 (;@4;) 2 (;@3;) 2 (;@3;)
            end
            local.get 7
            local.set 0
            i32.const 0
            local.set 7
            br 1 (;@3;)
          end
          local.get 7
          i32.const 1
          i32.shr_u
          local.set 0
          local.get 7
          i32.const 1
          i32.add
          i32.const 1
          i32.shr_u
          local.set 7
        end
        local.get 0
        i32.const 1
        i32.add
        local.set 0
        local.get 1
        i32.const 24
        i32.add
        i32.load
        local.set 8
        local.get 1
        i32.load offset=16
        local.set 6
        local.get 1
        i32.load offset=20
        local.set 9
        loop  ;; label = @3
          local.get 0
          i32.const -1
          i32.add
          local.tee 0
          i32.eqz
          br_if 2 (;@1;)
          local.get 9
          local.get 6
          local.get 8
          i32.load offset=16
          call_indirect (type 2)
          i32.eqz
          br_if 0 (;@3;)
        end
        i32.const 1
        return
      end
      local.get 1
      i32.load offset=20
      local.get 3
      local.get 2
      local.get 1
      i32.const 24
      i32.add
      i32.load
      i32.load offset=12
      call_indirect (type 1)
      return
    end
    i32.const 1
    local.set 0
    block  ;; label = @1
      local.get 9
      local.get 3
      local.get 2
      local.get 8
      i32.load offset=12
      call_indirect (type 1)
      br_if 0 (;@1;)
      i32.const 0
      local.set 0
      block  ;; label = @2
        loop  ;; label = @3
          block  ;; label = @4
            local.get 7
            local.get 0
            i32.ne
            br_if 0 (;@4;)
            local.get 7
            local.set 0
            br 2 (;@2;)
          end
          local.get 0
          i32.const 1
          i32.add
          local.set 0
          local.get 9
          local.get 6
          local.get 8
          i32.load offset=16
          call_indirect (type 2)
          i32.eqz
          br_if 0 (;@3;)
        end
        local.get 0
        i32.const -1
        i32.add
        local.set 0
      end
      local.get 0
      local.get 7
      i32.lt_u
      local.set 0
    end
    local.get 0)
  (func $_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$::fmt::h02a298d5b218d667 (type 2) (param i32 i32) (result i32)
    local.get 0
    i32.load
    local.get 1
    local.get 0
    i32.load offset=4
    i32.load offset=12
    call_indirect (type 2))
  (func $core::slice::index::slice_index_order_fail::h5c05174755728e22 (type 0) (param i32 i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 0
    i32.store
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    i32.const 8
    i32.add
    i32.const 12
    i32.add
    i64.const 2
    i64.store align=4
    local.get 3
    i32.const 32
    i32.add
    i32.const 12
    i32.add
    i32.const 1
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 66328
    i32.store offset=8
    local.get 3
    i32.const 1
    i32.store offset=36
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    local.get 3
    i32.const 4
    i32.add
    i32.store offset=40
    local.get 3
    local.get 3
    i32.store offset=32
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $core::panicking::panic_fmt::h78607b33a29a727d
    unreachable)
  (func $_$LT$core..alloc..layout..LayoutError$u20$as$u20$core..fmt..Debug$GT$::fmt::h424dac20ac50bdad (type 2) (param i32 i32) (result i32)
    local.get 1
    i32.load offset=20
    i32.const 66344
    i32.const 11
    local.get 1
    i32.const 24
    i32.add
    i32.load
    i32.load offset=12
    call_indirect (type 1))
  (func $alloc::raw_vec::finish_grow::hfa2e370a38b88d1e (type 6) (param i32 i32 i32 i32)
    (local i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        i32.const -1
        i32.le_s
        br_if 1 (;@1;)
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 3
              i32.load offset=4
              i32.eqz
              br_if 0 (;@5;)
              block  ;; label = @6
                local.get 3
                i32.const 8
                i32.add
                i32.load
                local.tee 4
                br_if 0 (;@6;)
                i32.const 0
                i32.load8_u offset=66436
                drop
                block  ;; label = @7
                  i32.const 0
                  i32.load offset=66440
                  local.tee 1
                  local.get 2
                  i32.add
                  local.tee 3
                  i32.const 0
                  i32.load offset=66444
                  i32.le_u
                  br_if 0 (;@7;)
                  local.get 2
                  i32.const 65535
                  i32.add
                  local.tee 3
                  i32.const 16
                  i32.shr_u
                  memory.grow
                  local.tee 1
                  i32.const -1
                  i32.eq
                  br_if 4 (;@3;)
                  i32.const 0
                  i32.load offset=66444
                  local.set 4
                  i32.const 0
                  local.get 1
                  i32.const 16
                  i32.shl
                  local.tee 1
                  local.get 3
                  i32.const -65536
                  i32.and
                  i32.add
                  i32.store offset=66444
                  i32.const 0
                  i32.load offset=66440
                  local.get 1
                  local.get 1
                  local.get 4
                  i32.eq
                  select
                  local.tee 1
                  local.get 2
                  i32.add
                  local.set 3
                end
                i32.const 0
                local.get 3
                i32.store offset=66440
                br 2 (;@4;)
              end
              local.get 3
              i32.load
              local.set 5
              block  ;; label = @6
                i32.const 0
                i32.load offset=66440
                local.tee 1
                local.get 2
                i32.add
                local.tee 3
                i32.const 0
                i32.load offset=66444
                i32.le_u
                br_if 0 (;@6;)
                local.get 2
                i32.const 65535
                i32.add
                local.tee 3
                i32.const 16
                i32.shr_u
                memory.grow
                local.tee 1
                i32.const -1
                i32.eq
                br_if 3 (;@3;)
                i32.const 0
                i32.load offset=66444
                local.set 6
                i32.const 0
                local.get 1
                i32.const 16
                i32.shl
                local.tee 1
                local.get 3
                i32.const -65536
                i32.and
                i32.add
                i32.store offset=66444
                i32.const 0
                i32.load offset=66440
                local.get 1
                local.get 1
                local.get 6
                i32.eq
                select
                local.tee 1
                local.get 2
                i32.add
                local.set 3
              end
              i32.const 0
              local.get 3
              i32.store offset=66440
              local.get 1
              i32.eqz
              br_if 2 (;@3;)
              local.get 1
              local.get 5
              local.get 4
              memory.copy
              br 1 (;@4;)
            end
            i32.const 0
            i32.load8_u offset=66436
            drop
            block  ;; label = @5
              i32.const 0
              i32.load offset=66440
              local.tee 1
              local.get 2
              i32.add
              local.tee 3
              i32.const 0
              i32.load offset=66444
              i32.le_u
              br_if 0 (;@5;)
              local.get 2
              i32.const 65535
              i32.add
              local.tee 3
              i32.const 16
              i32.shr_u
              memory.grow
              local.tee 1
              i32.const -1
              i32.eq
              br_if 2 (;@3;)
              i32.const 0
              i32.load offset=66444
              local.set 4
              i32.const 0
              local.get 1
              i32.const 16
              i32.shl
              local.tee 1
              local.get 3
              i32.const -65536
              i32.and
              i32.add
              i32.store offset=66444
              i32.const 0
              i32.load offset=66440
              local.get 1
              local.get 1
              local.get 4
              i32.eq
              select
              local.tee 1
              local.get 2
              i32.add
              local.set 3
            end
            i32.const 0
            local.get 3
            i32.store offset=66440
          end
          local.get 1
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 1
          i32.store offset=4
          local.get 0
          i32.const 8
          i32.add
          local.get 2
          i32.store
          local.get 0
          i32.const 0
          i32.store
          return
        end
        local.get 0
        i32.const 1
        i32.store offset=4
        local.get 0
        i32.const 8
        i32.add
        local.get 2
        i32.store
        local.get 0
        i32.const 1
        i32.store
        return
      end
      local.get 0
      i32.const 0
      i32.store offset=4
      local.get 0
      i32.const 8
      i32.add
      local.get 2
      i32.store
      local.get 0
      i32.const 1
      i32.store
      return
    end
    local.get 0
    i32.const 0
    i32.store offset=4
    local.get 0
    i32.const 1
    i32.store)
  (func $bytes::bytes::static_clone::hfe2f8292696dda85__.167_ (type 6) (param i32 i32 i32 i32)
    local.get 0
    i32.const 0
    i32.store offset=12
    local.get 0
    local.get 3
    i32.store offset=8
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 0
    i32.const 66392
    i32.store)
  (table (;0;) 19 19 funcref)
  (memory (;0;) 2)
  (global $__stack_pointer (mut i32) (i32.const 65536))
  (export "memory" (memory 0))
  (export "main" (func $main))
  (elem (;0;) (i32.const 1) func $core::fmt::num::imp::_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$::fmt::he696c0e431156bce $_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$::fmt::h02a298d5b218d667 $_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$::fmt::h743d7417ec5e8687 $bytes::bytes::static_clone::hfe2f8292696dda85 $bytes::bytes::static_to_vec::hb94cb51f4d0e0ea0 $bytes::bytes::static_drop::h58e729ae4cc2d39d $bytes::bytes::promotable_even_clone::h625046199c91628d $bytes::bytes::promotable_even_to_vec::h09f3a4302a0af998 $bytes::bytes::promotable_even_drop::hb0bc4b7b98503867 $bytes::bytes::promotable_odd_clone::h67309b0b4ba9becd $bytes::bytes::promotable_odd_to_vec::h76d8bdd4b86d6d4f $bytes::bytes::promotable_odd_drop::h237c4ebddc6329dc $core::ptr::drop_in_place$LT$core..alloc..layout..LayoutError$GT$::hc14ab9179ef63929 $_$LT$core..alloc..layout..LayoutError$u20$as$u20$core..fmt..Debug$GT$::fmt::h424dac20ac50bdad $bytes::bytes::shared_clone::h19b57f066b9013d5 $bytes::bytes::shared_to_vec::h0a07e611a75997a4 $bytes::bytes::shared_drop::h78acd3e84919f2f9 $bytes::bytes::static_clone::hfe2f8292696dda85__.167_)
  (data $.rodata (i32.const 65536) "library/alloc/src/raw_vec.rscapacity overflow\00\00\00\1c\00\01\00\11\00\00\00\00\00\01\00\1c\00\00\00\17\02\00\00\05\00\00\00memory allocation of  bytes failed\00\00H\00\01\00\15\00\00\00]\00\01\00\0d\00\00\00/Users/dmitry/.cargo/registry/src/index.crates.io-6f17d22bba15001f/bytes-1.5.0/src/bytes.rs\00\04\00\00\00\05\00\00\00\06\00\00\00\07\00\00\00\08\00\00\00\09\00\00\00\0a\00\00\00\0b\00\00\00\0c\00\00\00called `Result::unwrap()` on an `Err` value\00\0d\00\00\00\00\00\00\00\01\00\00\00\0e\00\00\00|\00\01\00[\00\00\00\03\04\00\002\00\00\00|\00\01\00[\00\00\00\11\04\00\00I\00\00\00\0f\00\00\00\10\00\00\00\11\00\00\00abort/Users/dmitry/.cargo/registry/src/index.crates.io-6f17d22bba15001f/bytes-1.5.0/src/lib.rs\00\00i\01\01\00Y\00\00\00s\00\00\00\09\00\00\00: \00\00X\03\01\00\00\00\00\00\d4\01\01\00\02\00\00\0000010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899 out of range for slice of length range end index \00\00\d2\02\01\00\10\00\00\00\b0\02\01\00\22\00\00\00slice index starts at  but ends at \00\f4\02\01\00\16\00\00\00\0a\03\01\00\0d\00\00\00LayoutErrorcodec/src/buffer.rs\00\003\03\01\00\13\00\00\00\a9\00\00\00\15\00\00\00\12\00\00\00\05\00\00\00\06\00\00\00sdk/src/evm.rs\00\00d\03\01\00\0e\00\00\00\9d\00\00\00\05\00\00\00"))
