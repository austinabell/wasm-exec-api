use super::ServerData;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::borrow::Cow;
use std::sync::Arc;
use tide::{Body, Response, StatusCode};
use utils::{load_wasm_module_recursive, wasm};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request<'a> {
    pub module_name: Cow<'a, str>,
    pub function_name: Cow<'a, str>,
    #[serde(default)]
    pub params: Vec<Number>,
}

pub async fn handle(mut req: tide::Request<Arc<ServerData>>) -> tide::Result {
    let Request {
        module_name,
        function_name,
        params,
    } = req.body_json().await?;
    let module = load_wasm_module_recursive(req.state().db.as_ref(), &module_name)?;

    let res = wasm::call_fn(&module, &function_name, params)?;
    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_json(&res)?)
        .build())
}
