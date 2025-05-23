pub mod github_fetcher;
pub mod gitlab_fetcher;
pub mod settings;

use async_trait::async_trait;
use downcast_rs::{Downcast, impl_downcast};
use gitlab::RestError;
use serde::Serialize;
use thiserror::Error;
use tokio::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GitHub(#[from] octocrab::Error),
    #[error(transparent)]
    GitLab(#[from] gitlab::GitlabError),
    #[error(transparent)]
    GitLabApi(#[from] gitlab::api::ApiError<RestError>),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[async_trait]
pub trait Fetcher: Downcast {
    async fn stars(&self) -> Result<u32>;
    fn project(&self) -> String;
}

impl_downcast!(Fetcher);
