(module
  (type (;0;) (func (param i32 i32)))
  (type (;1;) (func (param i32)))
  (type (;2;) (func))
  (import "fluentbase_v1alpha" "_sys_write" (func (;0;) (type 0)))
  (import "fluentbase_v1alpha" "_sys_halt" (func (;1;) (type 1)))
  (func (;2;) (type 2)
    i32.const 65536
    i32.const 12
    call 0
    i32.const 0
    call 1
    unreachable)
  (memory (;0;) 2)
  (global (;0;) (mut i32) (i32.const 65536))
  (export "memory" (memory 0))
  (export "main" (func 2))
  (data (;0;) (i32.const 65536) "Hello, World"))
