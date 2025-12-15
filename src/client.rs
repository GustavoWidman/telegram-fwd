use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use actix_web::web::Bytes;
use easy_config_store::ConfigStore;
use futures::Stream;
use grammers_client::{
    Client, Config, InitParams, InvocationError,
    session::Session,
    types::{Chat, Media},
};
use tokio::sync::RwLock;

use crate::{config, file};

#[derive(Hash)]
pub struct Cache {
    pub time: chrono::DateTime<chrono::Utc>,
    pub files: Vec<file::DownloadFile>,
}

pub struct ClientWrapper {
    pub config: Arc<ConfigStore<config::Config>>,
    desired_chats: Vec<Chat>,
    cached_files: RwLock<Option<Cache>>,
    client: RwLock<Option<Arc<Client>>>,
}

fn ask_code_to_user() -> String {
    let mut code = String::new();
    log::info!("Enter login code: ");
    std::io::stdin().read_line(&mut code).unwrap();
    code
}

impl ClientWrapper {
    pub async fn login(config: Arc<ConfigStore<config::Config>>) -> std::io::Result<Client> {
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
        .map_err(|e| std::io::Error::other(e.to_string()))?;

        if !client
            .is_authorized()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?
        {
            let token = client
                .request_login_code(&config.phone)
                .await
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            let code = ask_code_to_user();
            log::info!("Signing in...");
            client
                .sign_in(&token, &code)
                .await
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            client.session().save_to_file(&config.session_file_path)?;
            log::info!("Signed in!");
        }

        Ok(client)
    }

    pub async fn new(config: Arc<ConfigStore<config::Config>>) -> anyhow::Result<Self> {
        let client = Self::login(config.clone()).await?;

        let mut dialogs = client.iter_dialogs();
        let mut desired_chats: Vec<Chat> = vec![];
        while let Some(dialog) = dialogs
            .next()
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?
        {
            let chat = dialog.chat().clone();
            if chat.name().contains("Ubuntu Maniax") {
                desired_chats.push(chat);
            }
        }

        Ok(Self {
            config,
            desired_chats,
            cached_files: RwLock::new(None),
            client: RwLock::new(None),
        })
    }

    pub async fn get_files(&self) -> anyhow::Result<(Vec<file::DownloadFile>, u64)> {
        let cache = self.get_cached_files().await?;

        if let Some((files, hash)) = cache {
            return Ok((files, hash));
        }

        let client = self.reset_client().await?;

        let mut files: Vec<file::DownloadFile> = vec![];
        for chat in self.desired_chats.iter() {
            let mut messages = client.iter_messages(chat);
            while let Some(message) = messages.next().await? {
                if let Some(Media::Document(document)) = message.media() {
                    files.push(file::DownloadFile::new(
                        document.name().to_string(),
                        document.size(),
                        document,
                    ));
                }
            }
        }

        let hash = self.update_cache(files.clone()).await?;

        Ok((files, hash))
    }

    async fn get_cached_files(&self) -> anyhow::Result<Option<(Vec<file::DownloadFile>, u64)>> {
        let cached_files = self.cached_files.read().await;

        match cached_files.as_ref() {
            Some(cache) => {
                // invalidate cache if it's older than 15 minutes (play around with this value)
                if (chrono::Utc::now() - cache.time).num_seconds() > 15 * 60 {
                    log::info!("cache invalidated");

                    return Ok(None);
                };

                let mut hasher = DefaultHasher::new();
                cache.hash(&mut hasher);
                let hash = hasher.finish();

                Ok(Some((cache.files.clone(), hash)))
            }
            None => Ok(None),
        }
    }

    async fn update_cache(&self, files: Vec<file::DownloadFile>) -> anyhow::Result<u64> {
        let mut cache = self.cached_files.write().await;

        let entry = Cache {
            time: chrono::Utc::now(),
            files,
        };

        let mut hasher = DefaultHasher::new();
        entry.hash(&mut hasher);
        let hash = hasher.finish();

        cache.replace(entry);

        Ok(hash)
    }

    async fn get_client(&self) -> anyhow::Result<Arc<Client>> {
        let client = self.client.read().await;

        match client.as_ref() {
            Some(client) => Ok(client.clone()),
            None => self.reset_client().await,
        }
    }

    async fn reset_client(&self) -> anyhow::Result<Arc<Client>> {
        let mut client = self.client.write().await;

        let new_client = Arc::new(ClientWrapper::login(self.config.clone()).await?);
        client.replace(new_client.clone());

        Ok(new_client)
    }

    pub async fn download_file(
        wrapper: Arc<Self>,
        file: file::DownloadFile,
    ) -> anyhow::Result<impl Stream<Item = Result<Bytes, InvocationError>>> {
        let client = wrapper.get_client().await?;

        Ok(file.download_stream(client).await)
    }
}
