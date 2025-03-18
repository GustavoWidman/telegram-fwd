use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use easy_config_store::ConfigStore;

mod client;
mod config;
mod file;
mod logging;
mod serve;
mod utils;

pub struct AppState {
    client: Arc<client::ClientWrapper>,
    config: Arc<ConfigStore<config::Config>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logging::Logger::init(None);

    let config: ConfigStore<config::Config> = ConfigStore::read("config.toml")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let config = Arc::new(config);
    let client = Arc::new(
        client::ClientWrapper::new(config.clone())
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?,
    );

    let server_host = config.server_host.clone();
    let server_port = config.server_port;

    log::info!("Starting server on {}:{}", server_host, server_port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState {
                config: config.clone(),
                client: client.clone(),
            }))
            .service(serve::index)
            .service(serve::download)
            .service(serve::login)
    })
    .bind((server_host, server_port))?
    .run()
    .await
}
