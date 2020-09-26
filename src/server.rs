use super::wasm::execute_wasm;
use actix_web::{error, post, web, App, HttpServer, Result};
use dirs::home_dir;
use serde::Deserialize;
use serde_json::Number;
use sled::Db;
use std::borrow::Cow;
use std::sync::Arc;
use wasmer_runtime::{instantiate, ImportObject};

struct ServerData {
    /// Database which stores wasm code to be loaded and run.
    db: Arc<Db>,
}

#[derive(Deserialize, Debug)]
struct RequestPayload<'a> {
    wasm_hex: Cow<'a, str>,
    function_name: Cow<'a, str>,
    #[serde(default)]
    params: Vec<Number>,
    #[serde(default)]
    host_modules: Vec<Cow<'a, str>>,
}

#[post("/")]
async fn index(
    web::Json(RequestPayload {
        wasm_hex,
        function_name,
        params,
        host_modules,
    }): web::Json<RequestPayload<'_>>,
    data: web::Data<ServerData>,
) -> Result<String> {
    let wasm_bytes =
        hex::decode(wasm_hex.as_ref()).map_err(|e| error::ErrorBadRequest(e.to_string()))?;

    // Import host functions
    let mut imports = ImportObject::new();
    for module in host_modules {
        if let Some(bz) = data
            .db
            .get(module.as_ref())
            .map_err(|e| error::ErrorInternalServerError(e.to_string()))?
        {
            // TODO when host functions are stored, load them for the instance import
            let import = instantiate(bz.as_ref(), &ImportObject::new()).unwrap();
            imports.register(module, import);
        }
    }

    execute_wasm(&wasm_bytes, &function_name, params, &imports).map_err(error::ErrorNotAcceptable)
}

#[derive(Deserialize, Debug)]
struct RegisterPayload<'a> {
    module_name: Cow<'a, str>,
    wasm_hex: Cow<'a, str>,
}

#[post("/register")]
async fn register(
    web::Json(RegisterPayload {
        module_name,
        wasm_hex,
    }): web::Json<RegisterPayload<'_>>,
    data: web::Data<ServerData>,
) -> Result<String> {
    // TODO allow host function to be stored with sub functions
    let wasm_bytes =
        hex::decode(wasm_hex.as_ref()).map_err(|e| error::ErrorBadRequest(e.to_string()))?;
    data.db
        .insert(module_name.as_ref(), wasm_bytes)
        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
    Ok(format!("Stored {}", module_name))
}

/// Initialize database and start server.
pub(super) async fn start(port: u16) -> std::io::Result<()> {
    let data_dir = home_dir().unwrap();
    let db =
        Arc::new(sled::open(format!("{}/.wasm_exec_api", data_dir.to_str().unwrap())).unwrap());
    HttpServer::new(move || {
        App::new()
            .data(ServerData { db: db.clone() })
            // Executing wasm code
            .service(index)
            // Registering functions to be executed
            .service(register)
    })
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
