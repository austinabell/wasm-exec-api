use super::wasm::execute_wasm;
use actix_web::{error, post, web, App, HttpServer, Result};
use serde::Deserialize;
use serde_json::Number;
use std::borrow::Cow;
use wasmer_runtime::ImportObject;

#[derive(Deserialize, Debug)]
struct RequestPayload<'a> {
    wasm_hex: Cow<'a, str>,
    function_name: Cow<'a, str>,
    #[serde(default)]
    params: Vec<Number>,
    #[serde(default)]
    host_functions: Vec<Cow<'a, str>>,
}

struct ServerData {}

#[post("/")]
async fn index(
    web::Json(RequestPayload {
        wasm_hex,
        function_name,
        params,
        ..
    }): web::Json<RequestPayload<'_>>,
    _data: web::Data<ServerData>,
) -> Result<String> {
    let wasm_bytes =
        hex::decode(wasm_hex.as_ref()).map_err(|e| error::ErrorBadRequest(e.to_string()))?;

    // Import host functions
    let imports = ImportObject::new();
    // TODO register stored functions

    execute_wasm(&wasm_bytes, &function_name, params, &imports).map_err(error::ErrorNotAcceptable)
}

pub(super) async fn start(port: u16) -> std::io::Result<()> {
    HttpServer::new(|| App::new().data(ServerData {}).service(index))
        .bind(format!("127.0.0.1:{}", port))?
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
            "params": [2]
        }"#;
        let RequestPayload {
            wasm_hex,
            function_name,
            params,
            ..
        } = serde_json::from_str(req_payload).unwrap();
        assert_eq!(
            wasm_hex,
            "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b"
        );
        assert_eq!(function_name, "run");
        assert_eq!(params, [Number::from(2)]);
    }
}
