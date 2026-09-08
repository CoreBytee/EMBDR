use reqwest::Client;
use url::Url;

use tiktok_client::TiktokClient;

use crate::sources::{MediaAuthor, MediaData, MediaItem, Source};

pub struct TiktokSource {
    tiktok_client: TiktokClient,
}

impl TiktokSource {
    pub fn new() -> Self {
        let tiktok_client = match std::env::var("PROXY_URL") {
            Ok(proxy_url) if !proxy_url.is_empty() => {
                let proxy = reqwest::Proxy::all(&proxy_url).expect("Failed to parse PROXY_URL");
                let http_client = Client::builder()
                    .cookie_store(true)
                    .proxy(proxy)
                    .build()
                    .expect("Failed to build HTTP client with proxy");
                TiktokClient::new_with_http_client(http_client)
            }
            _ => TiktokClient::new().expect("Failed to create tiktok client"),
        };
        Self { tiktok_client }
    }
}

#[async_trait::async_trait]
impl Source for TiktokSource {
    fn name(&self) -> String {
        "TikTok".to_string()
    }

    fn id(&self) -> String {
        "tiktok".to_string()
    }

    fn short_id(&self) -> String {
        "tt".to_string()
    }

    fn color(&self) -> u32 {
        0xff004f
    }

    fn predicate(&self, url: &Url) -> bool {
        let Some(hostname) = url.host_str() else {
            return false;
        };

        let hostname_matches =
            hostname == "www.tiktok.com" || hostname == "tiktok.com" || hostname == "vm.tiktok.com";

        return hostname_matches;
    }

    async fn extract_media(
        &self,
        url: &Url,
    ) -> Result<crate::sources::MediaData, Box<dyn std::error::Error + Send + Sync>> {
        let tiktok_post = self.tiktok_client.fetch_from_url(url.as_str()).await?;
        println!("TikTok post: {:#?}", tiktok_post);

        let description = if tiktok_post.content.is_empty() {
            None
        } else {
            Some(tiktok_post.content.clone())
        };

        Ok(MediaData {
            id: tiktok_post.id.clone(),
            title: None,
            author: MediaAuthor {
                name: tiktok_post.account.display_name.clone(),
                url: format!("https://www.tiktok.com/@{}", tiktok_post.account.username),
            },
            community: None,
            description,
            items: tiktok_post
                .media_attachments
                .into_iter()
                .map(|item| MediaItem {
                    url: item.url.clone(),
                })
                .collect(),
            properties: vec![],
        })
    }
}
