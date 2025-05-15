use std::io;

use async_trait::async_trait;
use downcast_rs::{Downcast, impl_downcast};
use gitlab::RestError;
use thiserror::Error;

pub mod fetcher;
pub mod settings;

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

#[async_trait]
pub trait Fetcher: Downcast {
    async fn stars(&self) -> Result<u32>;
    fn project(&self) -> String;
}

impl_downcast!(Fetcher);
