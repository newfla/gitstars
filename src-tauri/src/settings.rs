use std::path::Path;

use crate::{
    Error, Fetcher, Result,
    fetcher::{github::GitHubFetcher, gitlab::GitLabFetcher},
};
use bon::Builder;
use getset::Getters;
use serde::{Deserialize, Serialize};
use setting_builder::{SetGitType, SetOwner, SetRepo};
use tokio::fs::{read_to_string, write};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GitType {
    GitHub,
    GitLab,
}

#[derive(Builder, Clone, Debug, Getters, Serialize, Deserialize)]
#[get = "pub"]
pub struct Setting {
    git_type: GitType,
    #[builder(into)]
    owner: String,
    #[builder(into)]
    repo: String,
    order: usize,
}

impl From<&Box<dyn Fetcher>> for SettingBuilder<SetRepo<SetOwner<SetGitType>>> {
    fn from(value: &Box<dyn Fetcher>) -> Self {
        if let Some(val) = value.downcast_ref::<GitHubFetcher>() {
            let project = val.project();
            let (owner, repo) = project.split_once('/').unwrap();
            return Setting::builder()
                .git_type(GitType::GitHub)
                .owner(owner)
                .repo(repo);
        }
        if let Some(val) = value.downcast_ref::<GitLabFetcher>() {
            let project = val.project();
            let (owner, repo) = project.split_once('/').unwrap();
            return Setting::builder()
                .git_type(GitType::GitLab)
                .owner(owner)
                .repo(repo);
        }
        unreachable!()
    }
}

impl From<Setting> for Box<dyn Fetcher> {
    fn from(value: Setting) -> Self {
        match value.git_type {
            GitType::GitHub => Box::new(
                GitHubFetcher::builder()
                    .owner(value.owner)
                    .repo(value.repo)
                    .build(),
            ),
            GitType::GitLab => Box::new(
                GitLabFetcher::builder()
                    .owner(value.owner)
                    .repo(value.repo)
                    .build(),
            ),
        }
    }
}

fn load(data: &str) -> Result<Vec<Setting>, Error> {
    let settings: Vec<Setting> = serde_json::from_str(data)?;
    Ok(settings)
}

fn load_fetchers(data: &str) -> Result<Vec<Box<dyn Fetcher>>, Error> {
    let settings = load(data)?;
    Ok(settings.into_iter().map(Into::into).collect())
}

#[allow(dead_code)]
pub async fn fetchers_from_path(path: &Path) -> Result<Vec<Box<dyn Fetcher>>> {
    let data = read_to_string(path).await?;
    load_fetchers(&data)
}

pub async fn settings_from_path(path: &Path) -> Result<Vec<Setting>, Error> {
    let data = read_to_string(path).await?;
    load(&data)
}

#[allow(dead_code)]
pub async fn store_fetchers_to_path(fetchers: &[Box<dyn Fetcher>], path: &Path) -> Result<()> {
    let settings: Vec<Setting> = fetchers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Into::<SettingBuilder<SetRepo<SetOwner<SetGitType>>>>::into(s)
                .order(i)
                .build()
        })
        .collect();
    store_settings_to_path(&settings, path).await
}

pub async fn store_settings_to_path(settings: &[Setting], path: &Path) -> Result<()> {
    let data = serde_json::to_string_pretty(settings)?;
    write(path, data).await?;
    Ok(())
}

mod test {
    #[test]
    fn test_load() {
        use crate::settings::load;

        let data = r#"[
            {
                "owner": "newfla",
                "repo": "diffusion-rs",
                "git_type": "GitHub",
                "order": 1
            },
            {
                "owner": "gitlab-org",
                "repo": "gitlab",
                "git_type": "GitLab",
                "order": 2
            }
            ]"#;
        let fetchers = load(data).unwrap();
        assert_eq!(2, fetchers.len())
    }

    #[tokio::test]
    async fn test_store() {
        use crate::settings::store_fetchers_to_path;
        use tempfile::NamedTempFile;

        use crate::Fetcher;
        use crate::fetcher::github::GitHubFetcher;
        use crate::fetcher::gitlab::GitLabFetcher;

        let gitlab = GitLabFetcher::builder()
            .owner("gitlab-org")
            .repo("gitlab")
            .build();
        let github = GitHubFetcher::builder()
            .owner("newfla")
            .repo("diffusion-rs")
            .build();
        let mut data: Vec<Box<dyn Fetcher>> = Vec::new();
        data.push(Box::new(gitlab));
        data.push(Box::new(github));

        let temp_file = NamedTempFile::new().unwrap();

        store_fetchers_to_path(&data, temp_file.path())
            .await
            .unwrap();
    }
}
