mod config;
mod logger;
mod server;
mod wasm;

use config::Config;
use serde::Deserialize;
use serde_json::Number;

#[derive(Deserialize, Debug)]
struct RequestPayload {
    wasm_hex: String,
    function_name: String,
    #[serde(default)]
    params: Vec<Number>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logger::setup_logger();

    let cfg: Config = argh::from_env();

    server::start(cfg.port).await
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
        } = serde_json::from_str(req_payload).unwrap();
        assert_eq!(
            wasm_hex,
            "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b"
        );
        assert_eq!(function_name, "run");
        assert_eq!(params, [Number::from(2)]);
    }
}
