mod handlers;
mod routes;

use crate::app::App;
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{web, App as ActixApp, HttpRequest, HttpServer};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

pub use routes::configure;

pub async fn serve(app: Arc<App>) -> std::io::Result<()> {
    let bind = app.config.bind_addr();
    let data = web::Data::new(app);

    tracing::info!(bind = %bind, "starting Actix-web server");

    HttpServer::new(move || {
        ActixApp::new()
            .wrap(TracingLogger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(data.clone())
            .configure(routes::configure)
            .service(Files::new("/assets", "static/assets").prefer_utf8(true))
            .default_service(web::route().to(spa_fallback))
    })
    .bind(&bind)?
    .run()
    .await
}

async fn spa_fallback(req: HttpRequest) -> actix_web::Result<actix_web::HttpResponse> {
    let path = req.path();
    if path.starts_with("/v1") || path == "/health" {
        return Ok(actix_web::HttpResponse::NotFound().finish());
    }
    Ok(NamedFile::open("static/index.html")?.into_response(&req))
}
