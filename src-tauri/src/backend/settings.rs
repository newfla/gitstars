use std::{collections::HashSet, hash::Hash, path::Path};

use crate::backend::{Error, Repo, Result};
use bon::Builder;
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use tokio::fs::{create_dir_all, read_to_string, write};
use uuid::Uuid;

#[derive(Builder, Clone, Debug, Eq, Getters, Serialize, Setters, Deserialize)]
#[get = "pub"]
pub struct Setting {
    id: Uuid,
    order: usize,
    #[getset(set = "pub")]
    favourite: bool,
    repo: Repo,
}

impl PartialEq for Setting {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
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
        self.id.hash(state);
    }
}

fn load_from_str(data: &str) -> Result<HashSet<Setting>, Error> {
    let settings: HashSet<Setting> = serde_json::from_str(data)?;
    Ok(settings)
}

pub async fn load(path: &Path) -> Result<HashSet<Setting>> {
    let data = read_to_string(path).await?;
    load_from_str(&data)
}

pub async fn store<'a, I>(settings: I, path: &Path) -> Result<()>
where
    I: IntoIterator<Item = &'a Setting> + Serialize,
{
    if let Some(parent) = path.parent() {
        create_dir_all(parent).await?;
    }
    let data = serde_json::to_string_pretty(&settings)?;
    write(path, data).await?;
    Ok(())
}

mod test {
    #[test]
    fn test_load() {
        use crate::backend::settings::load_from_str;

        let data = r#"[
            {
                "repo": {
                    "owner": "newfla",
                    "name": "diffusion-rs",
                    "git_type": "GitHub"
                },
                "order": 1,
                "favourite": true,
                "id": "3d69bebb-a800-4e6f-b318-1638a27b66c0"
            },
            {
                "repo": {
                    "owner": "gitlab-org",
                    "name": "gitlab",
                    "git_type": "GitLab"
                },
                "order": 2,
                "favourite": false,
                "id": "e70e6ca2-6e06-4f0c-8509-fea0645af5a1"
            }
            ]"#;
        let fetchers = load_from_str(data).unwrap();
        assert_eq!(2, fetchers.len())
    }

    #[tokio::test]
    async fn test_store() {
        use crate::backend::{
            Repo,
            settings::{Setting, store},
        };
        use tempfile::NamedTempFile;
        use uuid::Uuid;

        let gitlab_repo = Repo::builder()
            .owner("gitlab-org")
            .name("gitlab")
            .git_type(crate::backend::GitProvider::GitLab)
            .build();
        let github_repo = Repo::builder()
            .owner("newfla")
            .name("diffusion-rs")
            .git_type(crate::backend::GitProvider::GitHub)
            .build();
        let mut data = Vec::new();
        data.push(
            Setting::builder()
                .order(2)
                .id(Uuid::parse_str("e70e6ca2-6e06-4f0c-8509-fea0645af5a1").unwrap())
                .repo(gitlab_repo)
                .favourite(false)
                .build(),
        );
        data.push(
            Setting::builder()
                .order(1)
                .id(Uuid::parse_str("3d69bebb-a800-4e6f-b318-1638a27b66c0").unwrap())
                .repo(github_repo)
                .favourite(true)
                .build(),
        );

        let temp_file = NamedTempFile::new().unwrap();

        store(&data, temp_file.path()).await.unwrap();
    }
}
