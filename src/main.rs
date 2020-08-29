mod logger;
mod server;

use serde::Deserialize;
use serde_json::Number;
use std::num::ParseIntError;
use std::string::ToString;
use wasmer_runtime::{imports, instantiate, types::Type, DynFunc, Instance, Value as WasmValue};

#[derive(Deserialize, Debug)]
struct RequestPayload {
    wasm_hex: String,
    function_name: String,
    #[serde(default)]
    params: Vec<Number>,
}

fn call_fn(instance: &Instance, fn_name: &str, params: Vec<Number>) -> Result<String, String> {
    let function: DynFunc = instance.exports.get(&fn_name).map_err(|e| e.to_string())?;
    let sig_params = function.signature().params();

    let wasm_params = params_to_wasm(params, sig_params)?;

    match function.call(&wasm_params) {
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

fn params_to_wasm(values: Vec<Number>, types: &[Type]) -> Result<Vec<WasmValue>, String> {
    if values.len() != types.len() {
        return Err(format!(
            "Invalid parameter length, got {} and needed {}",
            values.len(),
            types.len(),
        ));
    }
    values
        .into_iter()
        .zip(types)
        .map(|(v, t)| match t {
            Type::I32 => Ok(WasmValue::I32(
                v.as_i64()
                    .ok_or_else(|| format!("Invalid type, expected I32, was {}", v))?
                    as i32,
            )),
            Type::I64 => {
                Ok(WasmValue::I64(v.as_i64().ok_or_else(|| {
                    format!("Invalid type, expected I64, was {}", v)
                })?))
            }
            Type::F32 => Ok(WasmValue::F32(
                v.as_f64()
                    .ok_or_else(|| format!("Invalid type, expected F32, was {}", v))?
                    as f32,
            )),
            Type::F64 => {
                Ok(WasmValue::F64(v.as_f64().ok_or_else(|| {
                    format!("Invalid type, expected F64, was {}", v)
                })?))
            }
            Type::V128 => Ok(WasmValue::V128(
                v.to_string()
                    .parse()
                    .map_err(|e: ParseIntError| e.to_string())?,
            )),
        })
        .collect()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logger::setup_logger();

    server::start().await
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
        } = serde_json::from_str(req_payload).unwrap();
        assert_eq!(
            wasm_hex,
            "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b"
        );
        assert_eq!(function_name, "run");
        assert_eq!(params, [Number::from(2)]);
    }
}
