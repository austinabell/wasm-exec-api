mod execute;
mod index;
mod register;

use crate::config::Config;
use actix_web::{App, HttpServer};
use dirs::home_dir;
use sled::Db;
use std::sync::Arc;

struct ServerData {
    /// Database which stores wasm code to be loaded and run.
    db: Arc<Db>,
}

/// Initialize database and start server.
pub(super) async fn start(
    Config {
        port,
        data_directory,
        memory,
    }: Config,
) -> std::io::Result<()> {
    let db = if memory {
        sled::Config::new().temporary(true).open().unwrap()
    } else {
        let path = data_directory
            .unwrap_or_else(|| format!("{}/.wasm_exec_api", home_dir().unwrap().to_str().unwrap()));
        sled::open(path).unwrap()
    };
    let db = Arc::new(db);
    HttpServer::new(move || {
        App::new()
            .data(ServerData { db: db.clone() })
            // Executing wasm code.
            .service(index::handle)
            // Registering functions to be executed.
            .service(register::handle)
            // Execute function on registered module.
            .service(execute::handle)
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header, test};
    use wasmer_runtime::Value as WasmValue;

    #[actix_rt::test]
    async fn full_usage_path() {
        let db = Arc::new(sled::Config::new().temporary(true).open().unwrap());

        let utils_code = include_bytes!("../../utils.wasm");
        let hex_utils = hex::encode(&utils_code);

        let linking_code = include_bytes!("../../linking.wasm");
        let hex_linking = hex::encode(&linking_code);

        const UTILS: &str = "utils";

        let mut app = test::init_service(
            App::new()
                .data(ServerData { db })
                .service(index::handle)
                .service(register::handle)
                .service(execute::handle),
        )
        .await;

        // Send request with no registered functions
        let req = test::TestRequest::post()
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .set_json(&index::Request {
                wasm_hex: hex_utils.as_str().into(),
                function_name: "double".into(),
                params: vec![2.into()],
                host_modules: Vec::new(),
            })
            .to_request();
        let resp: Vec<WasmValue> = test::read_response_json(&mut app, req).await;
        assert_eq!(resp, [WasmValue::I32(4)]);

        // Register utils module
        let req = test::TestRequest::post()
            .uri("/register")
            .header(header::CONTENT_TYPE, "application/json")
            .set_json(&register::Request {
                module_name: UTILS.into(),
                wasm_hex: hex_utils.as_str().into(),
                host_modules: Vec::new(),
            })
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());

        // Execute registered function
        let req = test::TestRequest::post()
            .uri("/execute")
            .header(header::CONTENT_TYPE, "application/json")
            .set_json(&execute::Request {
                module_name: UTILS.into(),
                function_name: "double".into(),
                params: vec![2.into()],
            })
            .to_request();
        let resp: Vec<WasmValue> = test::read_response_json(&mut app, req).await;
        assert_eq!(resp, [WasmValue::I32(4)]);

        // Send execute request with code linking to registered function
        let req = test::TestRequest::post()
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .set_json(&index::Request {
                wasm_hex: hex_linking.as_str().into(),
                function_name: "double_twice".into(),
                params: vec![2.into()],
                host_modules: vec!["utils".into()],
            })
            .to_request();
        let resp: Vec<WasmValue> = test::read_response_json(&mut app, req).await;
        assert_eq!(resp, [WasmValue::I32(8)]);
    }
}
