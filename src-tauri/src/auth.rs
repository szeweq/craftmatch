use std::{sync::{Arc, Mutex}, time};

use oauth2::{basic::{BasicClient, BasicTokenResponse}, reqwest::{self, header}, AccessToken, AuthUrl, ClientId, DeviceCode, DeviceCodeErrorResponse, DeviceCodeErrorResponseType, EndpointNotSet, EndpointSet, Scope, StandardDeviceAuthorizationResponse, TokenResponse, TokenUrl};
use serde::{Deserialize, Serialize};

const GH_CLIENT_ID: &str = "Ov23li3zsnysqjjjZLhm";

#[derive(Clone)]
pub struct GithubClient {
    auth: Arc<BasicClient<EndpointSet, EndpointSet, EndpointNotSet, EndpointNotSet, EndpointSet>>,
    http: reqwest::Client,
    token: Arc<Mutex<Option<AccessToken>>>,
}
impl GithubClient {
    pub fn setup() -> anyhow::Result<Self> {
        let auth = BasicClient::new(ClientId::new(GH_CLIENT_ID.to_string()))
            .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?)
            .set_token_uri(TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?)
            .set_device_authorization_url(oauth2::DeviceAuthorizationUrl::new("https://github.com/login/device/code".to_string())?);

        let http = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self { auth: Arc::new(auth), http, token: Arc::new(Mutex::new(None)) })
    }

    pub fn remove_token(&self) {
        *self.token.lock().unwrap() = None;
    }

    pub async fn authorize<F: Fn(&str, &str) -> anyhow::Result<()> + Send>(&self, send_code: F) -> anyhow::Result<()> {
        let details: StandardDeviceAuthorizationResponse = self.auth.exchange_device_code()
        .add_scope(Scope::new("user".to_string()))
        .add_scope(Scope::new("public_repo".to_string()))
        .request_async(&self.http).await?;

        let uri = details.verification_uri().to_string();

        opener::open_browser(&uri)?;
        send_code(details.user_code().secret(), &uri)?;

        let token_resp = gh_access_device_token(&self.http, details.device_code(), details.interval(), details.expires_in()).await?;
        let ref_token = &mut *self.token.lock().map_err(|_| anyhow::anyhow!("Failed to save token"))?;
        *ref_token = Some(token_resp.access_token().clone());

        Ok(())
    }

    fn token(&self) -> anyhow::Result<AccessToken> {
        self.token.lock().map_err(|_| anyhow::anyhow!("Failed to get token"))?.clone().ok_or_else(|| anyhow::anyhow!("no token"))
    }

    async fn gql_query(&self, query: &str) -> anyhow::Result<reqwest::Response> {
        let token = self.token()?;
        let resp = self.http.post("https://api.github.com/graphql").bearer_auth(token.secret())
            .header(header::USER_AGENT, header::HeaderValue::from_static("craftmatch/0.1.0"))
            .json(&Query { query })
            .send().await?;

        Ok(resp)
    }

    pub async fn user_info(&self) -> anyhow::Result<(Box<str>, Box<str>, u8)> {
        let resp = self.gql_query("query{viewer{name,avatarUrl},user(login:\"szeweq\"){viewerIsSponsoring,viewerIsFollowing},repository(owner:\"szeweq\",name:\"craftmatch\"){viewerHasStarred}}").await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get user info: {}", resp.text().await?));
        }
        let b = resp.bytes().await?;

        let Gql { data } = match serde_json::from_slice(&b) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Raw response: {}", String::from_utf8_lossy(&b));
                return Err(e.into());
            }
        };
        let viewer = map_get_val(&data, "viewer")?;
        let user = map_get_val(&data, "user")?;
        let repository = map_get_val(&data, "repository")?;
        let name = viewer.get("name").and_then(|v| v.as_str()).unwrap_or_default();
        let avatar_url = viewer.get("avatarUrl").and_then(|v| v.as_str()).unwrap_or_default();
        let is_sponsoring = user.get("viewerIsSponsoring").and_then(|v| v.as_bool()).unwrap_or_default();
        let is_following = user.get("viewerIsFollowing").and_then(|v| v.as_bool()).unwrap_or_default();
        let has_starred = repository.get("viewerHasStarred").and_then(|v| v.as_bool()).unwrap_or_default();
        Ok((name.into(), avatar_url.into(), ((is_sponsoring as u8) << 2) | ((is_following as u8) << 1) | (has_starred as u8)))
    }
}

#[allow(dead_code)]
fn default_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static("craftmatch/0.1.0"));
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/vnd.github+json"));
    headers.insert("X-GitHub-Api-Version", header::HeaderValue::from_static("2022-11-28"));
    headers
}

fn map_get_val<'a>(m: &'a serde_json::Map<String, serde_json::Value>, k: &str) -> anyhow::Result<&'a serde_json::Value> {
    m.get(k).ok_or_else(|| anyhow::anyhow!("Failed to get {} in {:?}", k, m))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum GitHubTokenResult {
    Ok(BasicTokenResponse),
    Err(DeviceCodeErrorResponse),
}
impl GitHubTokenResult {
    fn into_result(self) -> Result<BasicTokenResponse, DeviceCodeErrorResponse> {
        match self {
            Self::Ok(x) => Ok(x),
            Self::Err(x) => Err(x),
        }
    }
}

async fn gh_access_device_token(cli: &reqwest::Client, devcode: &DeviceCode, mut interval: time::Duration, expire: time::Duration) -> anyhow::Result<BasicTokenResponse> {
    let timeout_instant = std::time::Instant::now() + expire;

    loop {
        let now = std::time::Instant::now();
        if now >= timeout_instant {
            break Err(anyhow::anyhow!("Timed out waiting for access token."));
        }

        
        let resp = cli.post("https://github.com/login/oauth/access_token")
            .header(reqwest::header::ACCEPT, reqwest::header::HeaderValue::from_static("application/json"))
            .header(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"))
            .body(format!("client_id={}&device_code={}&grant_type=urn:ietf:params:oauth:grant-type:device_code", GH_CLIENT_ID, devcode.secret()))
            .send().await;

        if let Ok(xresp) = resp {
            if xresp.status().is_success() {
                let body: GitHubTokenResult = xresp.json().await?;
                match body.into_result() {
                    Ok(token) => break Ok(token),
                    Err(ser) => match ser.error() {
                        DeviceCodeErrorResponseType::AuthorizationPending => {},
                        DeviceCodeErrorResponseType::SlowDown => {
                            interval += time::Duration::from_secs(5);
                        }
                        _ => {
                            break Err(anyhow::anyhow!("Error: {}", ser));
                        }
                    }
                }
            }
        }

        tokio::time::sleep(interval).await;
    }
}

#[derive(Serialize)]
struct Query<'a> {
    query: &'a str
}

#[derive(Deserialize)]
struct Gql {
    data: serde_json::Map<String, serde_json::Value>
}