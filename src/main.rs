mod config;
mod logger;
mod server;
mod utils;
mod wasm;

use config::Config;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    logger::setup_logger();

    let cfg: Config = argh::from_env();

    server::start(cfg).await
}
