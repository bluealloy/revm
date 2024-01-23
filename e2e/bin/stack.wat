(module
  (type (;0;) (func (param i32)))
  (type (;1;) (func (result i32)))
  (type (;2;) (func))
  (import "fluentbase_v1alpha" "_sys_halt" (func $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5 (type 0)))
  (func $__get_stack_pointer (type 1) (result i32)
    global.get $__stack_pointer)
  (func $main (type 2)
    call $__get_stack_pointer
    call $fluentbase_sdk::bindings::_sys_halt::hbbb2822cabc581b5
    unreachable)
  (memory (;0;) 1)
  (global $__stack_pointer (mut i32) (i32.const 65536))
  (export "memory" (memory 0))
  (export "main" (func $main)))
