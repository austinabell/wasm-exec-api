mod execute;
mod index;
mod register;

use crate::LocalDB;
use actix_web::{App, HttpServer};
use std::sync::Arc;

struct ServerData {
    /// Database which stores wasm code to be loaded and run.
    db: Arc<LocalDB>,
}
/// Initialize database and start server.
pub(super) async fn start(port: u16, store: Arc<LocalDB>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data(ServerData { db: store.clone() })
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
    use crate::LocalDB;
    use actix_web::{http::header, test};
    use serde_cbor::{from_slice, to_vec};
    use utils::*;
    use wasmer_runtime::Value as WasmValue;

    #[actix_rt::test]
    async fn full_usage_path() {
        let db = Arc::new(LocalDB(sled::Config::new().temporary(true).open().unwrap()));

        let utils_code = include_bytes!("../../utils.wasm");
        let hex_utils = hex::encode(utils_code.as_ref());

        let linking_code = include_bytes!("../../linking.wasm");
        let hex_linking = hex::encode(linking_code.as_ref());

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
        let body = test::read_response(&mut app, req).await;
        let resp: [WasmValue; 1] = serde_json::from_slice(&body).unwrap();
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

    #[test]
    fn wasm_module_symmetric_serialize() {
        let wasm_ref = WasmModuleRef {
            code: b"test code",
            host_modules: &["one".into(), "two".into()],
        };
        let serialized = to_vec(&wasm_ref).unwrap();
        let wasm_mod_deser: WasmModule = from_slice(&serialized).unwrap();
        assert_eq!(wasm_mod_deser.code, wasm_ref.code);
        assert_eq!(wasm_mod_deser.host_modules, wasm_ref.host_modules);
    }

    #[test]
    fn store_load() {
        let config = sled::Config::new().temporary(true);
        let db = LocalDB(config.open().unwrap());
        let code = include_bytes!("../../utils.wasm");

        assert!(load_wasm_module_recursive(&db, "utils").is_err());

        // Trying to load with dependency module that doesn't exist
        assert!(store_wasm_module(&db, "test", code, &["utils".into()]).is_err());

        // Store and load utils
        store_wasm_module(&db, "utils", code, &[]).unwrap();
        assert!(load_wasm_module_recursive(&db, "utils").is_ok());

        // Shouldn't be able to overwrite existing module
        assert!(store_wasm_module(&db, "utils", code, &[]).is_err());

        // Should be able to store link with host module of now stored "utils"
        store_wasm_module(&db, "link", code, &["utils".into()]).unwrap();
        assert!(load_wasm_module_recursive(&db, "link").is_ok());
    }
}
