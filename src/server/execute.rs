use super::ServerData;
use crate::utils::*;
use crate::wasm::call_fn;
use actix_web::{error, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request<'a> {
    pub module_name: Cow<'a, str>,
    pub function_name: Cow<'a, str>,
    #[serde(default)]
    pub params: Vec<Number>,
}

#[post("/execute")]
async fn handle(
    web::Json(Request {
        module_name,
        function_name,
        params,
    }): web::Json<Request<'_>>,
    data: web::Data<ServerData>,
) -> Result<HttpResponse> {
    let module =
        load_wasm_module_recursive(&data.db, &module_name).map_err(error::ErrorNotAcceptable)?;

    let res = call_fn(&module, &function_name, params).map_err(error::ErrorNotAcceptable)?;

    Ok(HttpResponse::Ok().json(res))
}
