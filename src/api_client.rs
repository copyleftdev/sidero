use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

pub struct ApiClient;

impl ApiClient {
    pub async fn get_findings(token: &str, params: serde_json::Map<String, Value>) -> Result<Value> {
        let client = Client::new();
        
        let slug = Self::get_deployment_slug(&client, token).await?;
        
        let url = format!("https://semgrep.dev/api/v1/deployments/{}/findings", slug);
        
        let response = client
            .get(&url)
            .bearer_auth(token)
            .header("Accept", "application/json")
            .query(&params)
            .send()
            .await
            .context("Failed to send request to Semgrep Findings API")?;

        if !response.status().is_success() {
             let status = response.status();
             let text = response.text().await.unwrap_or_default();
             anyhow::bail!("API request failed with status {}: {}", status, text);
        }

        let json: Value = response.json().await.context("Failed to parse filings API response")?;
        Ok(json)
    }

    async fn get_deployment_slug(client: &Client, token: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct Deployment {
            slug: String,
        }
        #[derive(Deserialize)]
        struct DeploymentsResponse {
            deployments: Vec<Deployment>,
        }

        let url = "https://semgrep.dev/api/v1/deployments";
        let response = client
            .get(url)
            .bearer_auth(token)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to fetch deployments")?;

        if !response.status().is_success() {
             anyhow::bail!("Failed to fetch deployments: {}", response.status());
        }

        let data: DeploymentsResponse = response.json().await.context("Failed to parse deployments response")?;
        
        data.deployments.first()
            .map(|d| d.slug.clone())
            .ok_or_else(|| anyhow::anyhow!("No deployments found for this token"))
    }

    pub async fn fetch_url(url: &str) -> Result<String> {
        let client = Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .context(format!("Failed to fetch URL: {}", url))?;

        if !response.status().is_success() {
             anyhow::bail!("Request failed: {}", response.status());
        }

        let text = response.text().await.context("Failed to get response text")?;
        Ok(text)
    }
}
