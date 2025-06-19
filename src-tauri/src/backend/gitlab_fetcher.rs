use crate::backend::{GitProvider, Repo, Result};
use bon::builder;
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

#[builder]
pub async fn fetcher(repo: &Repo) -> Result<u32> {
    if repo.git_type != GitProvider::GitLab {
        return Err(super::Error::Wrongfetcher(
            repo.git_type.clone(),
            GitProvider::GitLab,
        ));
    }

    let client = GitlabBuilder::new_unauthenticated(GITLAB_URL)
        .build_async()
        .await?;
    let endpoint = projects::Project::builder()
        .project(repo.to_string())
        .statistics(true)
        .build()
        .unwrap();
    let res: Project = endpoint.query_async(&client).await?;
    Ok(res.star_count)
}

mod test {
    #[tokio::test]
    async fn test() {
        use crate::backend::{Repo, gitlab_fetcher::fetcher};

        let repo = Repo::builder()
            .git_type(crate::backend::GitProvider::GitLab)
            .owner("gitlab-org")
            .name("gitlab")
            .build();
        let stars = fetcher().repo(&repo).call().await;
        assert_eq!(repo.to_string(), "gitlab-org/gitlab");
        assert!(stars.is_ok())
    }
}
