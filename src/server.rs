use super::{call_wasm, RequestPayload};
use actix_web::{error, web, App, HttpServer, Result};

async fn index(req: web::Json<RequestPayload>) -> Result<String> {
    call_wasm(req.0).map_err(|e| error::ErrorNotAcceptable(e))
}

pub(super) async fn start() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
