(module
  (type (;0;) (func (param i32 i32)))
  (type (;1;) (func (param i32)))
  (type (;2;) (func))
  (import "fluentbase_v1alpha" "_sys_write" (func (;0;) (type 0)))
  (import "fluentbase_v1alpha" "_sys_halt" (func (;1;) (type 1)))
  (func (;2;) (type 2)
    i32.const 65536
    i32.const 178
    call 0
    i32.const 0
    call 1
    unreachable)
  (memory (;0;) 2)
  (global (;0;) (mut i32) (i32.const 65536))
  (export "memory" (memory 0))
  (export "deploy" (func 2))
  (data (;0;) (i32.const 65536) "\00asm\01\00\00\00\01\0d\03`\02\7f\7f\00`\01\7f\00`\00\00\02@\02\12fluentbase_v1alpha\0a_sys_write\00\00\12fluentbase_v1alpha\09_sys_halt\00\01\03\02\01\02\05\03\01\00\02\06\08\01\7f\01A\80\80\04\0b\07\11\02\06memory\02\00\04main\00\02\0a\1b\01\19\00A\80\80\84\80\00A\0c\10\80\80\80\80\00A\00\10\81\80\80\80\00\00\0b\0b\14\01\00A\80\80\04\0b\0cHello, World"))
