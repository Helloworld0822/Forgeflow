mod handlers;
mod routes;

use crate::app::App;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App as ActixApp, HttpServer};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

pub use routes::configure;

pub async fn serve(app: Arc<App>) -> std::io::Result<()> {
    let bind = app.config.bind_addr();
    let data = web::Data::from(app);

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
            .service(Files::new("/static", "static").prefer_utf8(true))
    })
    .bind(&bind)?
    .run()
    .await
}
