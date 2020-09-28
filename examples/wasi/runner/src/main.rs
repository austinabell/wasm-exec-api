use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "functions")]
extern "C" {
    fn fibonacci(n: u32) -> u32;
}

#[wasm_bindgen(module = "text")]
extern "C" {
    fn double(n: u32) -> u32;
}

fn main() {
    // Call rust generated wasm code
    let f = fibonacci(4);
    println!("fib 4 = {}", f);

    // Call WAT file generated wasm double function
    println!("doubled = {}", double(f));
}
