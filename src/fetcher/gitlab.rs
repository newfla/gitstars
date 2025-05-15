use crate::{Fetcher, Result};
use async_trait::async_trait;
use bon::Builder;
use gitlab::{
    GitlabBuilder,
    api::{AsyncQuery, projects},
};
use serde::Deserialize;

const GITLAB_URL: &str = "gitlab.com";

#[derive(Debug, Deserialize)]
struct Project {
    star_count: u32,
}

#[derive(Builder)]
pub struct GitLabFetcher {
    #[builder(into)]
    owner: String,
    #[builder(into)]
    repo: String,
}

#[async_trait]
impl Fetcher for GitLabFetcher {
    async fn stars(&self) -> Result<u32> {
        let client = GitlabBuilder::new_unauthenticated(GITLAB_URL)
            .build_async()
            .await?;
        let endpoint = projects::Project::builder()
            .project("gitlab-org/gitlab")
            .statistics(true)
            .build()
            .unwrap();
        let res: Project = endpoint.query_async(&client).await?;
        Ok(res.star_count)
    }

    fn project(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

mod test {
    #[tokio::test]
    async fn test() {
        use crate::{Fetcher, fetcher::gitlab::GitLabFetcher};

        let repo = GitLabFetcher::builder()
            .owner("gitlab-org")
            .repo("gitlab")
            .build();

        let stars = repo.stars().await;
        let name = repo.project();

        assert_eq!(name, "gitlab-org/gitlab");
        assert!(stars.is_ok())
    }
}
