mod config;
mod local_db;
mod logger;
mod server;

use config::Config;
use dirs::home_dir;
use local_db::LocalDB;
use std::sync::Arc;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logger::setup_logger();

    let Config {
        port,
        memory,
        data_directory,
    } = argh::from_env();

    let db = if memory {
        sled::Config::new().temporary(true).open().unwrap()
    } else {
        let path = data_directory
            .unwrap_or_else(|| format!("{}/.wasm_exec_api", home_dir().unwrap().to_str().unwrap()));
        sled::open(path).unwrap()
    };
    let db = Arc::new(LocalDB(db));

    server::start(port, db).await
}
