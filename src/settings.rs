use std::{
    fs::{read_to_string, write},
    path::Path,
};

use crate::{
    Error, Fetcher, Result,
    fetcher::{github::GitHubFetcher, gitlab::GitLabFetcher},
};
use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GitType {
    GitHub,
    GitLab,
}

#[derive(Builder, Serialize, Deserialize)]
struct Setting {
    git_type: GitType,
    #[builder(into)]
    owner: String,
    #[builder(into)]
    repo: String,
}

impl From<&Box<dyn Fetcher>> for Setting {
    fn from(value: &Box<dyn Fetcher>) -> Self {
        if let Some(val) = value.downcast_ref::<GitHubFetcher>() {
            let project = val.project();
            let (owner, repo) = project.split_once('/').unwrap();
            return Setting::builder()
                .git_type(GitType::GitHub)
                .owner(owner)
                .repo(repo)
                .build();
        }
        if let Some(val) = value.downcast_ref::<GitLabFetcher>() {
            let project = val.project();
            let (owner, repo) = project.split_once('/').unwrap();
            return Setting::builder()
                .git_type(GitType::GitLab)
                .owner(owner)
                .repo(repo)
                .build();
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

fn load(data: &str) -> Result<Vec<Box<dyn Fetcher>>, Error> {
    let settings: Vec<Setting> = serde_json::from_str(data)?;
    Ok(settings.into_iter().map(Into::into).collect())
}

pub fn load_from_path(path: &Path) -> Result<Vec<Box<dyn Fetcher>>> {
    let data = read_to_string(path)?;
    load(&data)
}

pub fn store_to_path(fetchers: &[Box<dyn Fetcher>], path: &Path) -> Result<()> {
    let settings: Vec<Setting> = fetchers.iter().map(Into::into).collect();
    let data = serde_json::to_string_pretty(&settings)?;
    write(path, data)?;
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
                "git_type": "GitHub"
            },
            {
                "owner": "gitlab-org",
                "repo": "gitlab",
                "git_type": "GitLab"
            }
            ]"#;
        let fetchers = load(data).unwrap();
        assert_eq!(2, fetchers.len())
    }

    #[test]
    fn test_store() {
        use crate::settings::store_to_path;
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

        store_to_path(&data, temp_file.path()).unwrap();
    }
}
