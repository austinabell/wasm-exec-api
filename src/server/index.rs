use super::ServerData;
use actix_web::{error, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::borrow::Cow;
use utils::{load_wasm_module_recursive, wasm::execute_wasm};
use wasmer_runtime::ImportObject;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request<'a> {
    pub wasm_hex: Cow<'a, str>,
    pub function_name: Cow<'a, str>,
    #[serde(default)]
    pub params: Vec<Number>,
    #[serde(default)]
    pub host_modules: Vec<Cow<'a, str>>,
}

#[post("/")]
async fn handle(
    web::Json(Request {
        wasm_hex,
        function_name,
        params,
        host_modules,
    }): web::Json<Request<'_>>,
    data: web::Data<ServerData>,
) -> Result<HttpResponse> {
    let wasm_bytes =
        hex::decode(wasm_hex.as_ref()).map_err(|e| error::ErrorBadRequest(e.to_string()))?;

    // Import host functions
    let mut imports = ImportObject::new();
    for module in host_modules {
        let import = load_wasm_module_recursive(data.db.as_ref(), module.as_ref())
            .map_err(error::ErrorInternalServerError)?;
        imports.register(module, import);
    }

    let res = execute_wasm(&wasm_bytes, &function_name, params, &imports)
        .map_err(error::ErrorNotAcceptable)?;
    Ok(HttpResponse::Ok().json(res))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn req_serialization() {
        let req_payload = r#"{
            "wasm_hex": "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b",
            "function_name": "run",
            "params": [2],
            "host_modules": ["utils"]
        }"#;
        let Request {
            wasm_hex,
            function_name,
            params,
            host_modules,
        } = serde_json::from_str(req_payload).unwrap();
        assert_eq!(
            wasm_hex,
            "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b"
        );
        assert_eq!(function_name, "run");
        assert_eq!(params, [Number::from(2)]);
        assert_eq!(host_modules, ["utils"]);
    }
}
