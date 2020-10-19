use crate::utils::{load_wasm_module_recursive, wasm::execute_wasm, WasmStore};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::borrow::Cow;
use std::sync::Arc;
use tide::{Body, Response, StatusCode};
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

pub async fn handle<S>(mut req: tide::Request<Arc<S>>) -> tide::Result
where
    S: WasmStore,
{
    let Request {
        wasm_hex,
        function_name,
        params,
        host_modules,
    } = req.body_json().await?;

    let wasm_bytes = hex::decode(wasm_hex.as_ref())?;

    // Import host functions
    let mut imports = ImportObject::new();
    for module in host_modules {
        let import = load_wasm_module_recursive(req.state().as_ref(), module.as_ref())?;
        imports.register(module, import);
    }

    let res = execute_wasm(&wasm_bytes, &function_name, params, &imports)?;
    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_json(&res)?)
        .build())
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
