(component
  (type (;0;) (func (param "a" u32)))
  (core module (;0;)
    (type (;0;) (func (param i32)))
    (type (;1;) (func (param i32 i32 i32 i32) (result i32)))
    (func (;0;) (type 0) (param i32)
      unreachable
    )
    (func (;1;) (type 1) (param i32 i32 i32 i32) (result i32)
      unreachable
    )
    (memory (;0;) 0)
    (export "foo" (func 0))
    (export "memory" (memory 0))
    (export "cabi_realloc" (func 1))
  )
  (core instance (;0;) (instantiate 0))
  (alias core export 0 "memory" (core memory (;0;)))
  (alias core export 0 "cabi_realloc" (core func (;0;)))
  (alias core export 0 "foo" (core func (;1;)))
  (func (;0;) (type 0) (canon lift (core func 1)))
  (export "foo" (func 0))
)