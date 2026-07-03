mod handlers;
mod routes;

use crate::app::App;
use crate::shutdown;
use actix_cors::Cors;
use actix_web::{web, App as ActixApp, HttpServer};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

pub use routes::configure;

const SHUTDOWN_TIMEOUT_SECS: u64 = 30;

pub async fn serve(app: Arc<App>) -> std::io::Result<()> {
    let bind = app.config.bind_addr();
    let data = web::Data::new(app);

    tracing::info!(bind = %bind, "starting API server");

    let server = HttpServer::new(move || {
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
    .shutdown_timeout(SHUTDOWN_TIMEOUT_SECS)
    .run();

    let handle = server.handle();
    tokio::spawn(async move {
        shutdown::wait_for_shutdown().await;
        tracing::info!("stopping API server (graceful shutdown)");
        handle.stop(true).await;
    });

    server.await
}
