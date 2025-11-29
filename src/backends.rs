use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use std::error::Error;

use reqwest::{Client, header};

use crate::cli::clone::RemoteRepo;

#[derive(Serialize, Deserialize)]
struct GitHubCreateRepositoryRequestBody {
    name: String,
    private: bool,
}

#[async_trait]
pub trait Backend {
    async fn is_existing_lockspec(&self, org: &str, name: &str)
    -> Result<bool, Box<dyn Error>>;
    async fn create_repository(&self, org: &str, name: &str) -> Result<(), Box<dyn Error>>;
    fn get_repo_info(&self, org: &str, repo: &str) -> RemoteRepo;
}

pub struct GitHubBackend<'a> {
    api_url: &'a str,
    client: Client,
}

#[async_trait]
impl Backend for GitHubBackend<'_> {
    async fn is_existing_lockspec(
        &self,
        org: &str,
        name: &str,
    ) -> Result<bool, Box<dyn Error>> {
        let resp = self
            .client
            .get(format!("{}/repos/{}/{}", self.api_url, org, name))
            .send()
            .await?
            .json::<HashMap<String, String>>()
            .await?;

        println!("{resp:?}");
        Ok(resp.contains_key("name"))
    }
    async fn create_repository(&self, org: &str, name: &str) -> Result<(), Box<dyn Error>> {
        let body = GitHubCreateRepositoryRequestBody {
            name: name.to_string(),
            private: true,
        };
        let status = self
            .client
            .post(format!("/orgs/{}/repos", org))
            .body(serde_json::to_string(&body)?)
            .send()
            .await?
            .status();

        if status.is_success() {
            Ok(())
        } else {
            Err(format!("Failed to create repository for {name}").into())
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

impl GitHubBackend<'_> {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let token = env::var_os("GITHUB_TOKEN")
            .ok_or(
                "No GITHUB_TOKEN found in the environment. Aborting."
            )?;
        let token_str = token
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
                format!("Bearer {token_str}").as_str()
            )?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            header::HeaderValue::from_static("2022-11-28"),
        );

        Ok(Self {
            api_url: "https://api.github.com/",
            client: Client::builder().default_headers(headers).build()?,
        })
    }
}

/// Get the currently configured araki backend.
pub fn get_current_backend() -> Result<impl Backend, Box<dyn Error>> {
    GitHubBackend::new()
}
