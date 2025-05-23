use std::{collections::HashSet, hash::Hash, path::Path};

use crate::backend::{
    Error, Fetcher, Result,
    {github_fetcher::GitHubFetcher, gitlab_fetcher::GitLabFetcher},
};
use bon::Builder;
use getset::Getters;
use serde::{Deserialize, Serialize};
use setting_builder::{SetGitType, SetOwner, SetRepo};
use tokio::fs::{create_dir_all, read_to_string, write};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GitType {
    GitHub,
    GitLab,
}

#[derive(Builder, Clone, Debug, Eq, Getters, Serialize, Deserialize)]
#[get = "pub"]
pub struct Setting {
    git_type: GitType,
    #[builder(into)]
    owner: String,
    #[builder(into)]
    repo: String,
    order: usize,
}

impl PartialEq for Setting {
    fn eq(&self, other: &Self) -> bool {
        self.git_type == other.git_type && self.owner == other.owner && self.repo == other.repo
    }
}

impl PartialOrd for Setting {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Setting {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}

impl Hash for Setting {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.git_type.hash(state);
        self.owner.hash(state);
        self.repo.hash(state);
    }
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

fn load(data: &str) -> Result<HashSet<Setting>, Error> {
    let settings: HashSet<Setting> = serde_json::from_str(data)?;
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

pub async fn settings_from_path(path: &Path) -> Result<HashSet<Setting>> {
    let data = read_to_string(path).await?;
    load(&data)
}

#[allow(dead_code)]
pub async fn store_fetchers_to_path(fetchers: &[Box<dyn Fetcher>], path: &Path) -> Result<()> {
    let settings: HashSet<Setting> = fetchers
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

pub async fn store_settings_to_path(settings: &HashSet<Setting>, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).await?;
    }
    let data = serde_json::to_string_pretty(settings)?;
    write(path, data).await?;
    Ok(())
}

mod test {
    #[test]
    fn test_load() {
        use crate::backend::settings::load;

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
        use crate::backend::settings::store_fetchers_to_path;
        use tempfile::NamedTempFile;

        use crate::Fetcher;
        use crate::backend::github_fetcher::GitHubFetcher;
        use crate::backend::gitlab_fetcher::GitLabFetcher;

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
