use anyhow::{anyhow, Error};
use serde_json::Number;
use std::string::ToString;
use wasmer_runtime::{
    instantiate, types::Type, DynFunc, ImportObject, Instance, Value as WasmValue,
};

/// Instantiates Wasm module and calls function name provided from the module.
pub fn execute_wasm(
    wasm_bytes: &[u8],
    function_name: &str,
    params: Vec<Number>,
    imports: &ImportObject,
) -> Result<Vec<WasmValue>, Error> {
    // Instantiate the wasm runtime
    let instance = instantiate(wasm_bytes, imports).map_err(|e| anyhow!("{}", e))?;

    call_fn(&instance, &function_name, params)
}

/// Calls the dynamic function with the params deserialized based on the function signature type.
pub fn call_fn(
    instance: &Instance,
    fn_name: &str,
    params: Vec<Number>,
) -> Result<Vec<WasmValue>, Error> {
    let function: DynFunc = instance.exports.get(&fn_name)?;
    let sig_params = function.signature().params();

    let wasm_params = params_to_wasm(params, sig_params)?;

    Ok(function.call(&wasm_params).map_err(|e| anyhow!("{}", e))?)
}

/// Converts the parameter values to the Wasmer value types to be used in execution.
fn params_to_wasm(values: Vec<Number>, types: &[Type]) -> Result<Vec<WasmValue>, Error> {
    if values.len() != types.len() {
        return Err(anyhow!(
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
                    .ok_or_else(|| anyhow!("Invalid type, expected I32, was {}", v))?
                    as i32,
            )),
            Type::I64 => {
                Ok(WasmValue::I64(v.as_i64().ok_or_else(|| {
                    anyhow!("Invalid type, expected I64, was {}", v)
                })?))
            }
            Type::F32 => Ok(WasmValue::F32(
                v.as_f64()
                    .ok_or_else(|| anyhow!("Invalid type, expected F32, was {}", v))?
                    as f32,
            )),
            Type::F64 => {
                Ok(WasmValue::F64(v.as_f64().ok_or_else(|| {
                    anyhow!("Invalid type, expected F64, was {}", v)
                })?))
            }
            Type::V128 => Ok(WasmValue::V128(v.to_string().parse()?)),
        })
        .collect()
}
