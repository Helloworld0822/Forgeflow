mod handlers;
mod routes;

use crate::app::App;
use actix_cors::Cors;
use actix_web::{web, App as ActixApp, HttpServer};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

pub use routes::configure;

pub async fn serve(app: Arc<App>) -> std::io::Result<()> {
    let bind = app.config.bind_addr();
    let data = web::Data::new(app);

    tracing::info!(bind = %bind, "starting API server");

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
    })
    .bind(&bind)?
    .run()
    .await
}
