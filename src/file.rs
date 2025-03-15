use actix_web::web::Bytes;
use async_stream::try_stream;
use futures::Stream;
use grammers_client::{
    Client, InvocationError,
    types::{Downloadable, Media, media::Document},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct DownloadFile {
    pub name: String,
    pub file_size: i64,
    downloadable: Downloadable,
}

impl DownloadFile {
    pub fn new(name: String, size: i64, document: Document) -> Self {
        Self {
            name,
            file_size: size,
            downloadable: Downloadable::Media(Media::Document(document)),
        }
    }

    pub fn download_stream(
        self,
        client: Arc<Client>,
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
