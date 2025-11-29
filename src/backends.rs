use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use reqwest::{RequestBuilder, Url};
use std::{collections::HashMap, env};
use std::error::Error;

use reqwest::{Client, header};

use crate::cli::clone::RemoteRepo;

#[derive(Serialize, Deserialize, Debug)]
struct GitHubCreateRepositoryRequestBody {
    name: String,
    private: bool,
}

#[async_trait]
pub trait Backend {
    async fn is_existing_lockspec(&self, org: &str, name: &str) -> Result<bool, BackendError>;
    async fn create_repository(&self, org: &str, name: &str) -> Result<(), BackendError>;
    fn get_repo_info(&self, org: &str, repo: &str) -> RemoteRepo;
    fn get(&self, path: &str) -> Result<RequestBuilder, BackendError>;
    fn post(&self, path: &str) -> Result<RequestBuilder, BackendError>;
}

pub struct GitHubBackend {
    api_url: Url,
    client: Client,
}

// An error type which is safe to send and share with other threads. Needed for async/await traits.
pub type BackendError = Box<dyn Error + Send + Sync>;

#[async_trait]
impl Backend for GitHubBackend {
    fn get(&self, path: &str) -> Result<RequestBuilder, BackendError> {
        Ok(
            self
                .client
                .get(self.api_url.join(path)?)
        )
    }
    fn post(&self, path: &str) -> Result<RequestBuilder, BackendError> {
        Ok(
            self
                .client
                .post(self.api_url.join(path)?)
        )
    }
    async fn is_existing_lockspec(
        &self,
        org: &str,
        name: &str,
    ) -> Result<bool, BackendError> {
        let resp = self
            .get(format!("/repos/{org}/{name}").as_str())?
            .send()
            .await?
            .json::<HashMap<String, String>>()
            .await?;

        Ok(resp.contains_key("name"))
    }
    async fn create_repository(&self, org: &str, name: &str) -> Result<(), BackendError> {
        let body = GitHubCreateRepositoryRequestBody {
            name: name.to_string(),
            private: true,
        };
        let result = self
            .post(format!("/orgs/{org}/repos").as_str())?
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        if result.status().is_success() {
            Ok(())
        } else {
            Err(result.text().await?.into())
        }
    }
    fn get_repo_info(&self, org: &str, repo: &str) -> RemoteRepo {
        RemoteRepo::new(
            Some(org.to_string()),
            repo.to_string(),
            Some("github.com".to_string()),
            Some("https://".to_string()),
        )
    }
}

impl GitHubBackend {
    pub fn new() -> Result<Self, BackendError> {
        let token = env::var_os("GITHUB_TOKEN")
            .ok_or(
                "No GITHUB_TOKEN found in the environment. Aborting."
            )?
            .into_string()
            .map_err(|err| {
                format!("Couldn't convert GITHUB_TOKEN to a string: {err:?}")
            })?;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(
                format!("Bearer {}", token.trim()).as_str()
            )?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            header::HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(
            "User-Agent",
            header::HeaderValue::from_static("araki"),
        );

        Ok(Self {
            api_url: Url::parse("https://api.github.com/")?,
            client: Client::builder().default_headers(headers).build()?,
        })
    }
}

/// Get the currently configured araki backend.
pub fn get_current_backend() -> Result<impl Backend, BackendError> {
    GitHubBackend::new()
}
