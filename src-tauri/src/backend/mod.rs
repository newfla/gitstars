pub mod github_fetcher;
pub mod gitlab_fetcher;
pub mod settings;

use std::fmt::Display;

use bon::Builder;
use bon::builder;
use getset::Getters;
use gitlab::RestError;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;
use tokio::io;

#[derive(Clone, Debug, Display, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GitType {
    GitHub,
    GitLab,
}

#[derive(Builder, Clone, Debug, Eq, PartialEq, Getters, Serialize, Deserialize)]
#[get = "pub"]
pub struct Repo {
    git_type: GitType,
    #[builder(into)]
    owner: String,
    #[builder(into)]
    project: String,
}

impl Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.project)
    }
}

impl Repo {
    pub async fn fetch(&self) -> Result<u32> {
        match self.git_type() {
            GitType::GitHub => github_fetcher::fetcher().repo(self).call().await,
            GitType::GitLab => gitlab_fetcher::fetcher().repo(self).call().await,
        }
    }
}

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
    #[error("Supported host: '{1}'. Get '{0}'")]
    Wrongfetcher(GitType, GitType),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
