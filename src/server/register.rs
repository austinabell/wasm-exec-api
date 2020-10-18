use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::Arc;
use utils::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request<'a> {
    pub module_name: Cow<'a, str>,
    pub wasm_hex: Cow<'a, str>,
    #[serde(default)]
    pub host_modules: Vec<Cow<'a, str>>,
}

pub async fn handle<S>(mut req: tide::Request<Arc<S>>) -> tide::Result<String>
where
    S: WasmStore,
{
    let Request {
        module_name,
        wasm_hex,
        host_modules,
    } = req.body_json().await?;

    let wasm_bytes = hex::decode(wasm_hex.as_ref())?;

    store_wasm_module(
        req.state().as_ref(),
        module_name.as_ref(),
        &wasm_bytes,
        &host_modules,
    )?;

    Ok(format!("Successfully stored module: {}", module_name))
}
