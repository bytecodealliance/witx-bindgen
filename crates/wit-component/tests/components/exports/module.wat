(module
  (memory (export "memory") 1)
  (func (export "cabi_realloc") (param i32 i32 i32 i32) (result i32) unreachable)
  (func (export "a") unreachable)
  (func (export "b") (param i32 i32 i32 i64) (result i32) unreachable)
  (func (export "c") (result i32) unreachable)
  (func (export "foo#a") unreachable)
  (func (export "foo#b") (param i32 i32) (result i32) unreachable)
  (func (export "foo#c") (param i32 i64 i32) (result i32) unreachable)
  (func (export "bar#a") (param i32) unreachable)
)