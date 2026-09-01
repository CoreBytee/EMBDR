use url::Url;

use reddit_client::RedditClient;

use crate::sources::{MediaAuthor, MediaData, MediaItem, MediaProperty, Source};

pub struct RedditSource {
    reddit_client: RedditClient,
}

impl RedditSource {
    pub fn new() -> Self {
        let reddit_client = RedditClient::new().expect("Failed to create reddit client");
        Self { reddit_client }
    }
}

#[async_trait::async_trait]
impl Source for RedditSource {
    fn name(&self) -> String {
        "Reddit".to_string()
    }

    fn id(&self) -> String {
        "reddit".to_string()
    }

    fn short_id(&self) -> String {
        "rd".to_string()
    }

    fn color(&self) -> u32 {
        0xFF4500
    }

    fn predicate(&self, url: &Url) -> bool {
        let Some(hostname) = url.host_str() else {
            return false;
        };

        let hostname_matches = hostname == "www.reddit.com"
            || hostname == "reddit.com"
            || hostname == "old.reddit.com"
            || hostname == "v.redd.it";

        let has_comments_path = url.path().contains("/comments/");

        return hostname_matches && has_comments_path;
    }

    async fn extract_media(
        &self,
        url: &Url,
    ) -> Result<MediaData, Box<dyn std::error::Error + Send + Sync>> {
        let reddit_post = self.reddit_client.fetch_from_url(url.as_str()).await?;

        let description = reddit_post.selftext.filter(|text| !text.is_empty());

        Ok(MediaData {
            id: reddit_post.id,
            author: MediaAuthor {
                name: format!("u/{}", reddit_post.author),
                url: format!("https://www.reddit.com/user/{}", reddit_post.author),
            },
            description,
            items: reddit_post
                .media_items
                .into_iter()
                .map(|item| MediaItem {
                    url: item.url().to_string(),
                })
                .collect(),
            properties: vec![
                // MediaProperty::LikeCount(reddit_post.score), // not in rss data
                MediaProperty::CommentCount(reddit_post.num_comments),
            ],
        })
    }
}
