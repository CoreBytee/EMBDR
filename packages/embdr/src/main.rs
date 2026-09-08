use std::sync::Arc;

use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::{
    Component, MessageFlags,
    component::{MediaGallery, MediaGalleryItem, UnfurledMediaItem},
};
use twilight_util::builder::message::{ContainerBuilder, TextDisplayBuilder};

use crate::{sources::Sources, utility::extract_urls::extract_urls};

mod sources;
mod utility;

struct EMBDR {
    sources: Arc<Sources>,
    gateway: Shard,
    http: Arc<HttpClient>,
}

impl EMBDR {
    fn new(token: String) -> Self {
        let sources = Arc::new(sources::get_sources());

        let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

        let gateway = Shard::new(ShardId::ONE, token.clone(), intents);
        let http = Arc::new(HttpClient::new(token.clone()));

        Self {
            sources,
            gateway,
            http,
        }
    }

    async fn poll_event(&mut self) -> bool {
        let Some(item) = self.gateway.next_event(EventTypeFlags::all()).await else {
            return false;
        };

        let Ok(event) = item else {
            tracing::warn!(source = ?item.unwrap_err(), "error receiving event");
            return true;
        };

        let sources = self.sources.clone();
        tokio::spawn(EMBDR::handle_event(event, sources, self.http.clone()));
        true
    }

    async fn handle_event(event: Event, sources: Arc<Sources>, http: Arc<HttpClient>) {
        match event {
            Event::MessageCreate(message) => {
                let urls = extract_urls(&message.content);
                let embeddable_urls = urls
                    .iter()
                    .filter(|url| sources.iter().any(|source| source.predicate(url)))
                    .collect::<Vec<_>>();

                if embeddable_urls.is_empty() {
                    return;
                }

                let mut components: Vec<Component> = Vec::new();

                let source = sources
                    .iter()
                    .find(|source| source.predicate(embeddable_urls[0]))
                    .expect("No source found for the embeddable URL (should not happen)");

                let media_data_result = source.extract_media(embeddable_urls[0]).await;
                if let Ok(media_data) = media_data_result {
                    println!("Media data: {:#?}", media_data);
                    if embeddable_urls.len() > 1 {
                        components.push(
                            ContainerBuilder::new()
                                .component(TextDisplayBuilder::new(":information_source: Multiple embeddable links found. Only showing the first embed.").build())
                                .build()
                                .into()
                        );
                    }

                    let mut container = ContainerBuilder::new().accent_color(Some(source.color()));

                    let header = match (&media_data.title, &media_data.community) {
                        (Some(title), Some(community)) => format!(
                            "## [{}]({}) · [{}]({})",
                            community.name, community.url,
                            title, media_data.author.url
                        ),
                        (Some(title), None) => format!(
                            "## [{}]({})",
                            title, media_data.author.url
                        ),
                        (None, Some(community)) => format!(
                            "## [{}]({}) · Post by [{}]({})",
                            community.name, community.url,
                            media_data.author.name, media_data.author.url
                        ),
                        (None, None) => format!(
                            "## Post by [{}]({})",
                            media_data.author.name, media_data.author.url
                        ),
                    };

                    container = container.component(TextDisplayBuilder::new(header).build());

                    if let Some(description) = media_data.description.clone() {
                        container =
                            container.component(TextDisplayBuilder::new(description).build());
                    }

                    if media_data.items.len() > 0 {
                        container = container.component(MediaGallery {
                            id: None,
                            items: media_data
                                .items
                                .iter()
                                .map(|item| MediaGalleryItem {
                                    description: None,
                                    spoiler: None,
                                    media: UnfurledMediaItem {
                                        url: item.url.clone(),
                                        proxy_url: None,
                                        content_type: None,
                                        width: None,
                                        height: None,
                                    },
                                })
                                .collect(),
                        });
                    }

                    if media_data.properties.len() > 0 {
                        let properties_text = media_data
                            .properties
                            .iter()
                            .map(|property| match property {
                                sources::MediaProperty::Score(count) => {
                                    format!("{} {}", property.emoji(), count)
                                }
                                sources::MediaProperty::LikeCount(count) => {
                                    format!("{} {}", property.emoji(), count)
                                }
                                sources::MediaProperty::CommentCount(count) => {
                                    format!("{} {}", property.emoji(), count)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" • ");

                        container =
                            container.component(TextDisplayBuilder::new(properties_text).build());
                    }

                    components.push(container.build().into());
                } else {
                    components.push(
                        ContainerBuilder::new()
                            .component(
                                TextDisplayBuilder::new(
                                    ":warning: Failed to extract media from the provided link.",
                                )
                                .build(),
                            )
                            .build()
                            .into(),
                    );

                    tracing::error!(
                        error = ?media_data_result.unwrap_err(),
                        "Failed to extract media from the provided link"
                    );
                }

                let result = http
                    .create_message(message.channel_id)
                    .reply(message.id)
                    .flags(MessageFlags::IS_COMPONENTS_V2 | MessageFlags::SUPPRESS_NOTIFICATIONS)
                    .components(&components)
                    .await;

                if let Err(err) = result {
                    tracing::error!(error = ?err, "Failed to send embed message");
                }
            }
            Event::Ready(event) => {
                tracing::info!("Gateway is ready. Logged in as {}", event.user.name);
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Load env vars
    dotenvy::dotenv().expect("Failed to load environment variables from .env file");

    // Install the ring crypto provider for rustls (required by twilight)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls CryptoProvider");

    // Load EMBDR
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set in .env file");
    let mut embdr = EMBDR::new(token.clone());
    while embdr.poll_event().await {}
}
