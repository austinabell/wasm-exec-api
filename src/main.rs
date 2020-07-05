use actix_web::{error, web, App, HttpServer, Result};
use serde::Deserialize;
use std::string::ToString;
use wasmer_runtime::{imports, instantiate, Func, Instance};
// use wasmer_runtime_core::types::WasmExternType;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum WasmParam {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    TupleTwo(Box<WasmParam>, Box<WasmParam>),
    TupleThree(Box<WasmParam>, Box<WasmParam>, Box<WasmParam>),
}

#[derive(Deserialize)]
struct RequestPayload {
    wasm_hex: String,
    function_name: String,
    params: Option<WasmParam>,
    return_type: Option<String>,
}

fn call_fn(
    instance: &Instance,
    fn_name: &str,
    params: Option<WasmParam>,
    return_type: &str,
) -> Result<String, String> {
    use WasmParam::*;
    match (&params, return_type) {
        (None, "i32") => {
            let function: Func<(), i32> =
                instance.exports.get(&fn_name).map_err(|e| e.to_string())?;
            function
                .call()
                .map(|r| r.to_string())
                .map_err(|e| e.to_string())
        }
        (Some(I32(i)), "i32") => {
            let function: Func<i32, i32> =
                instance.exports.get(&fn_name).map_err(|e| e.to_string())?;

            function
                .call(*i)
                .map(|r| r.to_string())
                .map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "params: {:?}, return_type: {:?} not supported",
            params, return_type
        )),
    }
}

fn call_wasm(
    RequestPayload {
        wasm_hex,
        function_name,
        params,
        return_type,
    }: RequestPayload,
) -> Result<String, String> {
    let wasm_bytes = hex::decode(&wasm_hex).map_err(|e| e.to_string())?;

    // Instantiate the wasm runtime
    let instance = instantiate(&wasm_bytes, &imports! {}).map_err(|e| e.to_string())?;

    let return_type = if let Some(ty) = return_type {
        ty
    } else {
        return Ok("".to_owned());
    };

    let res = call_fn(&instance, &function_name, params, &return_type)?;

    // Convert result into string to return
    Ok(res.to_string())
}

async fn index(req: web::Json<RequestPayload>) -> Result<String> {
    call_wasm(req.0).map_err(|e| error::ErrorNotAcceptable(e))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
