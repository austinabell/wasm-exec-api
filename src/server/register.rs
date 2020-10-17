use super::ServerData;
use actix_web::{error, post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use utils::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request<'a> {
    pub module_name: Cow<'a, str>,
    pub wasm_hex: Cow<'a, str>,
    #[serde(default)]
    pub host_modules: Vec<Cow<'a, str>>,
}

#[post("/register")]
async fn handle(
    web::Json(Request {
        module_name,
        wasm_hex,
        host_modules,
    }): web::Json<Request<'_>>,
    data: web::Data<ServerData>,
) -> Result<HttpResponse> {
    let wasm_bytes = hex::decode(wasm_hex.as_ref()).map_err(error::ErrorBadRequest)?;

    store_wasm_module(
        data.db.as_ref(),
        module_name.as_ref(),
        &wasm_bytes,
        &host_modules,
    )
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().body(format!("Successfully stored module: {}", module_name)))
}
