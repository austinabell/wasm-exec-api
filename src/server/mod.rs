pub mod execute;
pub mod index;
pub mod register;

use std::sync::Arc;
use tide::utils::After;
use tide::Response;
use utils::WasmStore;

/// Initialize database and start server.
pub async fn start<S>(port: u16, store: Arc<S>) -> tide::Result<()>
where
    S: WasmStore + Send + Sync + 'static,
{
    let mut app = tide::with_state(store);

    app.with(After(|mut res: Response| async {
        // ! You may want to remove this error message, only helpful for debugging
        if let Some(s) = res.error().map(|e| e.to_string()) {
            res.set_body(s);
        }
        Ok(res)
    }));

    app.at("/").post(index::handle);
    app.at("/register").post(register::handle);
    app.at("/execute").post(execute::handle);
    app.listen(format!("localhost:{}", port)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::LocalDB;
    use async_std::prelude::*;
    use async_std::task;
    use serde_cbor::{from_slice, to_vec};
    use std::time::Duration;
    use utils::*;
    use wasmer_runtime::Value as WasmValue;

    #[async_std::test]
    async fn full_usage_path() {
        let db = Arc::new(LocalDB(sled::Config::new().temporary(true).open().unwrap()));

        let utils_code = include_bytes!("../../utils.wasm");
        let hex_utils = hex::encode(utils_code.as_ref());

        let linking_code = include_bytes!("../../linking.wasm");
        let hex_linking = hex::encode(linking_code.as_ref());

        const UTILS: &str = "utils";

        let port = portpicker::pick_unused_port().unwrap();
        let server = task::spawn(async move {
            let mut app = tide::with_state(db);

            app.at("/").post(index::handle);
            app.at("/register").post(register::handle);
            app.at("/execute").post(execute::handle);

            app.listen(("localhost", port)).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let uri = format!("http://localhost:{}", port);
            let mut res = surf::post(uri)
                .body(http_types::Body::from_json(&index::Request {
                    wasm_hex: hex_utils.as_str().into(),
                    function_name: "double".into(),
                    params: vec![2i32.into()],
                    host_modules: Vec::new(),
                })?)
                .await?;
            assert_eq!(res.status(), http_types::StatusCode::Ok);
            let value: [WasmValue; 1] = res.body_json().await.unwrap();
            assert_eq!(value, [WasmValue::I32(4)]);

            // Register utils module
            let uri = format!("http://localhost:{}/register", port);
            let res = surf::post(uri)
                .body(http_types::Body::from_json(&register::Request {
                    module_name: UTILS.into(),
                    wasm_hex: hex_utils.as_str().into(),
                    host_modules: Vec::new(),
                })?)
                .await?;
            assert_eq!(res.status(), http_types::StatusCode::Ok);

            // Execute registered function
            let uri = format!("http://localhost:{}/execute", port);
            let mut res = surf::post(uri)
                .body(http_types::Body::from_json(&execute::Request {
                    module_name: UTILS.into(),
                    function_name: "double".into(),
                    params: vec![2i32.into()],
                })?)
                .await?;
            assert_eq!(res.status(), http_types::StatusCode::Ok);
            let value: [WasmValue; 1] = res.body_json().await.unwrap();
            assert_eq!(value, [WasmValue::I32(4)]);

            // Send execute request with code linking to registered function
            let uri = format!("http://localhost:{}", port);
            let mut res = surf::post(uri)
                .body(http_types::Body::from_json(&index::Request {
                    wasm_hex: hex_linking.as_str().into(),
                    function_name: "double_twice".into(),
                    params: vec![2i32.into()],
                    host_modules: vec!["utils".into()],
                })?)
                .await?;
            assert_eq!(res.status(), http_types::StatusCode::Ok);
            let value: [WasmValue; 1] = res.body_json().await.unwrap();
            assert_eq!(value, [WasmValue::I32(8)]);

            Ok(())
        });

        server.race(client).await.unwrap();
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

    #[async_std::test]
    async fn store_load() {
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
