use crate::{Fetcher, Result};
use async_trait::async_trait;
use bon::Builder;

#[derive(Builder)]
pub struct GitHubFetcher {
    #[builder(into)]
    owner: String,
    #[builder(into)]
    repo: String,
}

#[async_trait]
impl Fetcher for GitHubFetcher {
    async fn stars(&self) -> Result<u32> {
        let client = octocrab::instance();
        let repo = client.repos(&self.owner, &self.repo).get().await?;
        Ok(repo.stargazers_count.unwrap_or_default())
    }

    fn project(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

mod test {
    #[tokio::test]
    async fn test() {
        use crate::{Fetcher, fetcher::github::GitHubFetcher};

        let repo = GitHubFetcher::builder()
            .owner("newfla")
            .repo("diffusion-rs")
            .build();

        let stars = repo.stars().await;
        let name = repo.project();

        assert_eq!(name, "newfla/diffusion-rs");
        assert!(stars.is_ok())
    }
}
