# EMBDR

EMBDR is a Discord bot written in Rust that provides better embeds for Instagram links. When a user posts an Instagram link in a Discord server, EMBDR detects it and replies with a custom embed that displays a playable video and improved metadata.

## Features

- Detects Instagram links in Discord messages
- Replies with a custom embed containing a playable video, author information, and post statistics
- Extensible source system for adding support for additional platforms

## Project Structure

EMBDR is organised as a Cargo workspace with two packages:

- **`packages/embdr`** — The main Discord bot. Connects to the Discord gateway, listens for messages, and replies with rich embeds.
- **`packages/instagram-client`** — A standalone Rust library for interacting with Instagram's GraphQL API. Handles CSRF token management, shortcode extraction, and media fetching.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024)

## Installation

1. **Clone the repository:**
   ```sh
   git clone https://github.com/CoreBytee/EMBDR.git
   cd EMBDR
   ```

2. **Build the project:**
   ```sh
   cargo build
   ```

## Configuration

Create a `.env` file in the project root with the following variables:

```
DISCORD_TOKEN=your_discord_bot_token
PROXY_URL=your_proxy_url
```

The `PROXY_URL` variable is optional. When set, all Instagram API requests will be routed through the specified proxy.

## Running

Start the bot with:

```sh
cargo run
```

The bot will log in to Discord and start listening for messages.

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository
2. Create a new branch for your feature or bugfix
3. Make your changes
4. Open a pull request describing your changes

Please follow the existing code style and include clear commit messages.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Publishing a New Release

There is a workflow in this repository to publish a new release. To activate it, create a new tag and push it to GitHub:

```sh
git tag v1.0.0-beta.5
git push origin --tags
```

This will trigger the publish release workflow, which builds the binary and creates a GitHub release with the compiled artifact.
