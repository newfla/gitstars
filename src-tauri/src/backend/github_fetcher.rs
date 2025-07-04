use crate::backend::{GitProvider, Repo, Result};
use bon::builder;

#[builder]
pub async fn fetcher(repo: &Repo) -> Result<u32> {
    if repo.git_type != GitProvider::GitHub {
        return Err(super::Error::WrongFetcher(
            repo.git_type.clone(),
            GitProvider::GitHub,
        ));
    }
    let client = octocrab::instance();
    let repo = client.repos(&repo.owner, &repo.name).get().await?;
    Ok(repo.stargazers_count.unwrap_or_default())
}

mod test {
    #[ignore]
    #[tokio::test]
    async fn test() {
        use crate::backend::{Repo, github_fetcher::fetcher};

        let repo = Repo::builder()
            .git_type(crate::backend::GitProvider::GitHub)
            .owner("newfla")
            .name("diffusion-rs")
            .build();
        let stars = fetcher().repo(&repo).call().await;
        assert_eq!(repo.to_string(), "newfla/diffusion-rs");
        assert!(stars.is_ok())
    }
}
