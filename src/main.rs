use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use easy_config_store::ConfigStore;
use grammers_client::{
    Client, Config, InitParams,
    session::Session,
    types::{Chat, Media},
};

mod config;
mod file;
mod logging;
mod serve;
mod utils;

fn ask_code_to_user() -> String {
    let mut code = String::new();
    log::info!("Enter login code: ");
    std::io::stdin().read_line(&mut code).unwrap();
    code
}

pub struct AppState {
    client: Arc<Client>,
    files: Arc<Vec<file::DownloadFile>>,
    config: Arc<ConfigStore<config::Config>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logging::Logger::init(None);

    let config: ConfigStore<config::Config> = ConfigStore::read("config.toml")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let client = Client::connect(Config {
        session: Session::load_file_or_create(&config.session_file_path)?,
        api_id: config.api_id,
        api_hash: config.api_hash.clone(),
        params: InitParams {
            // Fetch the updates we missed while we were offline
            catch_up: true,
            ..Default::default()
        },
    })
    .await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    if !client
        .is_authorized()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
    {
        let token = client
            .request_login_code(&config.phone)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let code = ask_code_to_user();
        log::info!("Signing in...");
        client
            .sign_in(&token, &code)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        client.session().save_to_file(&config.session_file_path)?;
        log::info!("Signed in!");
    }

    let mut dialogs = client.iter_dialogs();
    let mut desired_chats: Vec<Chat> = vec![];
    while let Some(dialog) = dialogs
        .next()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
    {
        let chat = dialog.chat().clone();
        if chat.name().contains("Ubuntu Maniax") {
            desired_chats.push(chat);
        }
    }

    let mut files: Vec<file::DownloadFile> = vec![];
    for chat in desired_chats {
        let mut messages = client.iter_messages(&chat);
        while let Some(message) = messages
            .next()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        {
            if let Some(Media::Document(document)) = message.media() {
                files.push(file::DownloadFile::new(
                    document.name().to_string(),
                    document.size(),
                    document,
                ));
            }
        }
    }

    let client = Arc::new(client);
    let files = Arc::new(files);
    let config = Arc::new(config);

    let server_host = config.server_host.clone();
    let server_port = config.server_port;

    log::info!("Starting server on {}:{}", server_host, server_port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState {
                config: config.clone(),
                client: client.clone(),
                files: files.clone(),
            }))
            .service(serve::index)
            .service(serve::download)
            .service(serve::login)
    })
    .bind((server_host, server_port))?
    .run()
    .await
}
