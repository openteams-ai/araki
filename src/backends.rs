use async_trait::async_trait;
use console::style;
use reqwest::{ClientBuilder, RequestBuilder, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::time::Duration;
use tokio::time;

use reqwest::{Client, header};

use crate::cli::clone::RemoteRepo;
use crate::common::get_araki_dir;

#[derive(Serialize, Deserialize, Debug)]
struct GitHubCreateRepositoryRequestBody {
    name: String,
    private: bool,
}

#[async_trait]
pub trait Backend {
    /// Check if a lockspec exists under an org.
    ///
    /// * `org`: Organization containing lockspec repositories
    /// * `name`: Name of the lockspec to search for
    async fn is_existing_lockspec(&self, org: &str, name: &str) -> Result<bool, BackendError>;

    /// Create a repository under an org.
    ///
    /// * `org`: Organization where the repository should be created
    /// * `name`: Name of the repository (i.e. the name of the lockspec)
    async fn create_repository(&self, org: &str, name: &str) -> Result<(), BackendError>;

    /// Log in to the backend.
    async fn login(&self) -> Result<(), BackendError>;

    /// Build a RemoteRepo containing repository information.
    ///
    /// * `org`: Organization
    /// * `repo`: Repository name
    fn get_repo_info(&self, org: &str, repo: &str) -> RemoteRepo;

    /// Create an authenticated GET request builder.
    ///
    /// * `path`: Suffix to join with the API url to send the request to
    fn get(&self, path: &str) -> Result<RequestBuilder, BackendError>;

    /// Create an authenticated GET request builder.
    ///
    /// * `path`: Suffix to join with the API url to send the request to
    fn post(&self, path: &str) -> Result<RequestBuilder, BackendError>;
}

pub struct GitHubBackend {
    api_url: Url,
    client: Option<Client>,
}

// An error type which is safe to send and share with other threads. Needed for async/await traits.
pub type BackendError = Box<dyn Error + Send + Sync>;

#[async_trait]
impl Backend for GitHubBackend {
    fn get(&self, path: &str) -> Result<RequestBuilder, BackendError> {
        Ok(self
            .client
            .as_ref()
            .ok_or("Please authenticate with `araki auth login` before continuing.")?
            .get(self.api_url.join(path)?))
    }
    fn post(&self, path: &str) -> Result<RequestBuilder, BackendError> {
        Ok(self
            .client
            .as_ref()
            .ok_or("Please authenticate with `araki auth login` before continuing.")?
            .post(self.api_url.join(path)?))
    }
    async fn is_existing_lockspec(&self, org: &str, name: &str) -> Result<bool, BackendError> {
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

    /// Log the user in.
    /// See https://docs.github.com/en/enterprise-cloud@latest/apps/creating-github-apps/writing-code-for-a-github-app/building-a-cli-with-a-github-app
    /// for the reference followed here.
    async fn login(&self) -> Result<(), BackendError> {
        let resp = Self::request_device_code().await?;

        println!(
            "{}{}",
            style("Please visit: ").bold().yellow(),
            resp.verification_uri
        );
        println!(
            "{}{}",
            style("and enter code: ").bold().yellow(),
            resp.user_code
        );

        Self::poll_for_token(&resp.device_code, Duration::from_secs(resp.interval)).await
    }
}

#[derive(Deserialize, Debug)]
struct GitHubDeviceCodeResponse {
    verification_uri: String,
    user_code: String,
    device_code: String,
    interval: u64,
}

impl GitHubBackend {
    const CLIENT_ID: &str = "Ov23liFxqmYL2jVV2QZ0"; // araki OAuth app

    /// Create a new set of authenticated headers.
    ///
    /// * `token`: Token to use for authentication with the API
    fn make_authenticated_request_headers(token: &str) -> Result<header::HeaderMap, BackendError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            header::HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(format!("Bearer {}", token.trim()).as_str())?,
        );
        headers.insert("User-Agent", header::HeaderValue::from_static("araki"));
        Ok(headers)
    }

    /// Create a new GitHubBackend.
    pub fn new() -> Result<Self, BackendError> {
        let mut client = None;
        if let Some(token) = Self::get_cached_token() {
            let builder = ClientBuilder::new();
            client = Some(
                builder
                    .default_headers(Self::make_authenticated_request_headers(&token)?)
                    .build()?,
            )
        }

        Ok(Self {
            api_url: Url::parse("https://api.github.com/")?,
            client,
        })
    }

    /// Return the cached token, if possible.
    /// The token is stored at ~/.araki/araki-token
    fn get_cached_token() -> Option<String> {
        fs::read_to_string(get_araki_dir().ok()?.join("araki-token")).ok()
    }

    /// Request a device code to use to initiate authentication with the GH API.
    async fn request_device_code() -> Result<GitHubDeviceCodeResponse, BackendError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        let client = Client::builder().default_headers(headers).build()?;

        let url = Url::parse_with_params(
            "https://github.com/login/device/code",
            &[("client_id", Self::CLIENT_ID), ("scope", "repo admin:org")],
        )?;

        let response = client
            .post(url)
            .send()
            .await?
            .error_for_status()?
            .json::<GitHubDeviceCodeResponse>()
            .await?;
        Ok(response)
    }

    /// Request a token for the given device code.
    ///
    /// * `device_code`: Device code to use to authenticate
    async fn request_token(device_code: &str) -> Result<serde_json::Value, BackendError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        let client = Client::builder().default_headers(headers).build()?;

        let url = Url::parse_with_params(
            "https://github.com/login/oauth/access_token",
            &[
                ("client_id", Self::CLIENT_ID),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ],
        )?;

        Ok(client
            .post(url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?)
    }

    /// Poll the GH API while waiting for the user to allow araki to grab a user token.
    ///
    /// * `device_code`: Device code to use to authenticate
    /// * `interval`: Interval to poll the API at. If the API says to slow down, we automatically
    ///   add a 5s delay to this before the next poll
    async fn poll_for_token(device_code: &str, interval: Duration) -> Result<(), BackendError> {
        loop {
            let response = match Self::request_token(device_code).await {
                Ok(resp) => resp,
                Err(err) => {
                    eprintln!("ERROR: {err}");
                    std::process::exit(1);
                }
            };
            let error = response.get("error");

            match error {
                Some(val) if val == "authorization_pending" => {
                    time::sleep(interval).await;
                }
                Some(val) if val == "slow_down" => {
                    time::sleep(interval + Duration::from_secs(5)).await;
                }
                Some(val) if val == "expired_token" => {
                    return Err(
                        "The GitHub token araki uses has expired. Please run `login` again.".into(),
                    );
                }
                Some(val) if val == "access_denied" => {
                    return Err("Login cancelled by user.".into());
                }
                Some(err) => {
                    return Err(format!("Error getting araki github app token: {err}").into());
                }
                None => {
                    // Write the new token to the araki-token file
                    let token = serde_json::from_value::<String>(
                        response
                            .get("access_token")
                            .ok_or("Unexpected response whil getting a GitHub user access token")?
                            .clone(),
                    )?;
                    let mut file = fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(get_araki_dir()?.join("araki-token"))?;
                    writeln!(file, "{}", token)?;
                    return Ok(());
                }
            }
        }
    }
}

/// Get the currently configured araki backend.
pub fn get_current_backend() -> Result<impl Backend, BackendError> {
    GitHubBackend::new()
}
