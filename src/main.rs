use actix_web::{error, web, App, HttpServer, Result};
use log::LevelFilter;
use serde::Deserialize;
use std::string::ToString;
use wasmer_runtime::{imports, instantiate, DynFunc, Instance, Value as WasmValue};

fn setup_logger() {
    let mut logger_builder = pretty_env_logger::formatted_timed_builder();
    if let Ok(s) = ::std::env::var("RUST_LOG") {
        logger_builder.parse_filters(&s);
    } else {
        logger_builder.filter(None, LevelFilter::Info);
    }
    let logger = logger_builder.build();
    async_log::Logger::wrap(logger, || 12)
        .start(log::LevelFilter::Trace)
        .unwrap();
}

#[derive(Deserialize, Debug)]
struct RequestPayload {
    wasm_hex: String,
    function_name: String,
    #[serde(default)]
    params: Vec<WasmValue>,
}

fn call_fn(instance: &Instance, fn_name: &str, params: Vec<WasmValue>) -> Result<String, String> {
    let function: DynFunc = instance.exports.get(&fn_name).map_err(|e| e.to_string())?;
    match function.call(&params) {
        Ok(r) => serde_json::to_string(&r).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn call_wasm(
    RequestPayload {
        wasm_hex,
        function_name,
        params,
    }: RequestPayload,
) -> Result<String, String> {
    let wasm_bytes = hex::decode(&wasm_hex).map_err(|e| e.to_string())?;

    // Instantiate the wasm runtime
    let instance = instantiate(&wasm_bytes, &imports! {}).map_err(|e| e.to_string())?;

    call_fn(&instance, &function_name, params)
}

async fn index(req: web::Json<RequestPayload>) -> Result<String> {
    call_wasm(req.0).map_err(|e| error::ErrorNotAcceptable(e))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    setup_logger();

    HttpServer::new(|| App::new().route("/", web::post().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn req_serialization() {
        let req_payload = r#"{
            "wasm_hex": "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b",
            "function_name": "run",
            "params": [{"I32": 2}]
        }"#;
        let RequestPayload {
            wasm_hex,
            function_name,
            params,
        } = serde_json::from_str(req_payload).unwrap();
        assert_eq!(
            wasm_hex,
            "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b"
        );
        assert_eq!(function_name, "run");
        assert_eq!(params, [WasmValue::I32(2)]);
    }
}
