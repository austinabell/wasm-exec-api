(module
  ;; Import the double function from the utils wasm code.
  (import "utils" "double" (func $double (param i32) (result i32)))
  
  (func (export "double_twice") (param i32) (result i32)
    local.get 0
    call $double
    call $double
  )
)
