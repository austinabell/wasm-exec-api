use actix_web::{error, web, App, HttpServer, Result};
use serde::Deserialize;
use std::string::ToString;
use wasmer_runtime::{imports, instantiate, Func};

#[derive(Deserialize)]
struct RequestPayload {
    wasm_hex: String,
    function_name: String,
}

async fn call_wasm(wasm_hex: &str, function_name: &str) -> Result<String, String> {
    let wasm_bytes = hex::decode(&wasm_hex).map_err(|e| e.to_string())?;

    // Instantiate the wasm runtime
    let instance = instantiate(&wasm_bytes, &imports! {}).map_err(|e| e.to_string())?;

    // Setup function
    let function: Func<i32, i32> = instance
        .exports
        .get(&function_name)
        .map_err(|e| e.to_string())?;

    // Call function
    let result = function.call(42).map_err(|e| e.to_string())?;

    // Convert result into string to return
    Ok(result.to_string())
}

async fn index(req: web::Json<RequestPayload>) -> Result<String> {
    call_wasm(&req.wasm_hex, &req.function_name)
        .await
        .map_err(|e| error::ErrorNotAcceptable(e))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
