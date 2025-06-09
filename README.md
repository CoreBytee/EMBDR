# EMBDR

EMBDR is a Discord bot and webserver that provides better embeds for Instagram links. When a user posts an Instagram link in a Discord server, EMBDR detects it and replies with a custom embed that displays a playable video and improved metadata.

## Features

- Detects Instagram links in Discord messages
- Replies with a custom embed containing a playable video and post information
- Provides a web endpoint for rich embeds and video redirection

## Installation

1. **Clone the repository:**
   ```sh
   git clone https://github.com/CoreBytee/EMBDR.git
   cd EMBDR
   ```

2. **Install dependencies:**
   ```sh
   bun install
   ```

3. **Configure environment variables:**
   - Copy `.env.example` to `.env` (if available) or create a `.env` file with the following variables:
     ```
     DISCORD_TOKEN=your_discord_bot_token
     WEBSERVER_PORT=3000
     WEBSERVER_URL=https://your.domain.com
     ```
   - Replace the values as needed.

## Running the Bot

Start the bot and webserver with:

```sh
bun run start
```

The bot will log in to Discord and start listening for messages. The webserver will run on the port specified in your `.env`.

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository
2. Create a new branch for your feature or bugfix
3. Make your changes
4. Open a pull request describing your changes

Please follow the existing code style and include clear commit messages.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.