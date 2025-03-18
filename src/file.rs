use actix_web::web::Bytes;
use async_stream::try_stream;
use futures::Stream;
use grammers_client::{
    InvocationError,
    types::{Downloadable, Media, media::Document},
};
use std::sync::Arc;

use crate::client::ClientWrapper;

#[derive(Clone)]
pub struct DownloadFile {
    pub name: String,
    pub file_size: i64,
    random_id: i64,
    document_id: i64,
    downloadable: Downloadable,
}

impl std::hash::Hash for DownloadFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.file_size.hash(state);
        self.random_id.hash(state);
        self.document_id.hash(state);
    }
}

impl DownloadFile {
    pub fn new(name: String, size: i64, document: Document) -> Self {
        Self {
            name,
            file_size: size,
            random_id: rand::random(),
            document_id: document.id(),
            downloadable: Downloadable::Media(Media::Document(document)),
        }
    }

    pub fn download_stream(
        self,
        client: Arc<ClientWrapper>,
    ) -> impl Stream<Item = Result<Bytes, InvocationError>> {
        try_stream! {
            let mut download = client.iter_download(&self.downloadable);
            while let Some(chunk) = download.next().await? {
                // Convert the chunk into actix_web::web::Bytes and yield it.
                yield Bytes::from(chunk);
            }
        }
    }
}
