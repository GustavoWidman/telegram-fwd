use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::RwLock,
};

use grammers_client::{
    Client,
    types::{Chat, Media},
};

use crate::file;

#[derive(Hash)]
pub struct Cache {
    pub time: chrono::DateTime<chrono::Utc>,
    pub files: Vec<file::DownloadFile>,
}

pub struct ClientWrapper {
    pub client: Client,
    desired_chats: Vec<Chat>,
    cached_files: RwLock<Option<Cache>>,
}

impl ClientWrapper {
    pub async fn new(client: Client) -> std::io::Result<Self> {
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

        Ok(Self {
            client,
            desired_chats,
            cached_files: RwLock::new(None),
        })
    }

    pub async fn get_files(&self) -> std::io::Result<(Vec<file::DownloadFile>, u64)> {
        let cache = self.get_cached_files().await?;

        if let Some((files, hash)) = cache {
            return Ok((files, hash));
        }

        let mut files: Vec<file::DownloadFile> = vec![];
        for chat in self.desired_chats.iter() {
            let mut messages = self.client.iter_messages(chat);
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

        let hash = self.update_cache(files.clone()).await?;

        Ok((files, hash))
    }

    async fn get_cached_files(&self) -> std::io::Result<Option<(Vec<file::DownloadFile>, u64)>> {
        let cached_files = self
            .cached_files
            .read()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        match cached_files.as_ref() {
            Some(cache) => {
                // invalidate cache if it's older than 15 minutes (play around with this value)
                if (chrono::Utc::now() - cache.time).num_seconds() > 60 * 15 {
                    return Ok(None);
                };

                let mut hasher = DefaultHasher::new();
                cache.hash(&mut hasher);
                let hash = hasher.finish();

                return Ok(Some((cache.files.clone(), hash)));
            }
            None => Ok(None),
        }
    }

    async fn update_cache(&self, files: Vec<file::DownloadFile>) -> std::io::Result<u64> {
        let mut cache = self
            .cached_files
            .write()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

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
}

impl Deref for ClientWrapper {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
