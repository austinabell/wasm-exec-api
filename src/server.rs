use super::utils::*;
use super::wasm::execute_wasm;
use actix_web::{error, post, web, App, HttpServer, Result};
use dirs::home_dir;
use serde::Deserialize;
use serde_json::Number;
use sled::Db;
use std::borrow::Cow;
use std::sync::Arc;
use wasmer_runtime::ImportObject;

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
        let import = load_wasm_module_recursive(&data.db, module.as_ref())
            .map_err(error::ErrorInternalServerError)?;
        imports.register(module, import);
    }

    execute_wasm(&wasm_bytes, &function_name, params, &imports).map_err(error::ErrorNotAcceptable)
}

#[derive(Deserialize, Debug)]
struct RegisterPayload<'a> {
    module_name: Cow<'a, str>,
    wasm_hex: Cow<'a, str>,
    #[serde(default)]
    host_modules: Vec<Cow<'a, str>>,
}

#[post("/register")]
async fn register(
    web::Json(RegisterPayload {
        module_name,
        wasm_hex,
        host_modules,
    }): web::Json<RegisterPayload<'_>>,
    data: web::Data<ServerData>,
) -> Result<String> {
    let wasm_bytes = hex::decode(wasm_hex.as_ref()).map_err(error::ErrorBadRequest)?;

    store_wasm_module(&data.db, module_name.as_ref(), &wasm_bytes, &host_modules)
        .map_err(error::ErrorInternalServerError)?;

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
            "params": [2],
            "host_modules": ["utils"]
        }"#;
        let RequestPayload {
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
