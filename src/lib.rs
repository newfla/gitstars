use gitlab::RestError;
use thiserror::Error;

pub mod fetcher;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GitHub(#[from] octocrab::Error),
    #[error(transparent)]
    GitLab(#[from] gitlab::GitlabError),
    #[error(transparent)]
    GitLabApi(#[from] gitlab::api::ApiError<RestError>),
}

pub trait Fetcher {
    fn get_stars(&self) -> impl std::future::Future<Output = Result<u32>> + Send;
    fn project(&self) -> String;
}
