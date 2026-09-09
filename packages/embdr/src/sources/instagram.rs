use reqwest::Client;
use url::Url;

use instagram_client::InstagramClient;

use crate::sources::{MediaAuthor, MediaData, MediaItem, MediaProperty, Source};

pub struct InstagramSource {
    instagram_client: InstagramClient,
}

impl InstagramSource {
    pub fn new() -> Self {
        let instagram_client = match std::env::var("PROXY_URL") {
            Ok(proxy_url) if !proxy_url.is_empty() => {
                let proxy = reqwest::Proxy::all(&proxy_url).expect("Failed to parse PROXY_URL");
                let http_client = Client::builder()
                    .cookie_store(true)
                    .proxy(proxy)
                    .build()
                    .expect("Failed to build HTTP client with proxy");
                InstagramClient::new_with_http(http_client)
            }
            _ => InstagramClient::new().expect("Failed to create instagram client"),
        };
        Self { instagram_client }
    }
}

#[async_trait::async_trait]
impl Source for InstagramSource {
    fn name(&self) -> String {
        "Instagram".to_string()
    }

    fn id(&self) -> String {
        "instagram".to_string()
    }

    fn short_id(&self) -> String {
        "ig".to_string()
    }

    fn color(&self) -> u32 {
        0xE1306C
    }

    fn predicate(&self, url: &Url) -> bool {
        let Some(hostname) = url.host_str() else {
            return false;
        };

        let hostname_matches = hostname == "www.instagram.com"
            || hostname == "instagram.com"
            || hostname == "ddinstagram.com";

        return hostname_matches;
    }

    async fn extract_media(
        &self,
        url: &Url,
    ) -> Result<crate::sources::MediaData, Box<dyn std::error::Error + Send + Sync>> {
        let instagram_post = self.instagram_client.fetch_from_url(url.as_str()).await?;
        println!("Instagram post: {:#?}", instagram_post);

        Ok(MediaData {
            id: instagram_post.shortcode.clone(),
            title: None,
            author: MediaAuthor {
                name: instagram_post.owner.username.clone(),
                url: format!(
                    "https://www.instagram.com/{}/",
                    instagram_post.owner.username
                ),
            },
            community: None,
            description: Some(instagram_post.caption.clone()).filter(|s| !s.is_empty()),
            items: instagram_post
                .media_items
                .into_iter()
                .map(|item| MediaItem {
                    url: item.url().into(),
                })
                .collect(),
            properties: vec![
                MediaProperty::LikeCount(instagram_post.like_count),
                MediaProperty::CommentCount(instagram_post.comment_count),
            ],
        })
    }
}
