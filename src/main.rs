use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use clap::Parser;

use crate::config::{Config, config};

mod cli;
mod client;
mod config;
mod file;
mod logging;
mod serve;
mod utils;

pub struct AppState {
    client: Arc<client::ClientWrapper>,
    config: Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = cli::Args::parse();
    logging::Logger::init(args.verbosity);

    let config = config(args.config).map_err(|e| std::io::Error::other(e.to_string()))?;

    let client = Arc::new(
        client::ClientWrapper::new(config.clone())
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?,
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
            .service(serve::health_check)
            .service(serve::login)
    })
    .bind((server_host, server_port))?
    .run()
    .await
}
