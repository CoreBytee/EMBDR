import { Client } from "discord.js";
import getUrls from "get-urls";
import { DISCORD_TOKEN, WEBSERVER_PORT, WEBSERVER_URL } from "./env";
import InstagramGraph from "./graphs/instagram";

const sources = [await import("./sources/instagram")];

const discord = new Client({ intents: [53608447] });
await discord.login(DISCORD_TOKEN);

discord.on("ready", () => {
	console.log("Discord bot is ready");
});

discord.on("messageCreate", async (message) => {
	if (message.author.bot) return;
	if (!("guild" in message.channel)) return;

	const content = message.content;
	const urls = getUrls(content);

	for (const urlString of urls) {
		const url = new URL(urlString);
		for (const source of sources) {
			const match = await source.matchUrl(url);
			if (!match) continue;

			message.suppressEmbeds(true);
			source.reply(url, message);
			return;
		}
	}
});

console.log("Starting webserver at", WEBSERVER_URL, "on port", WEBSERVER_PORT);
Bun.serve({
	port: WEBSERVER_PORT,
	routes: {
		"/e/ig/:id": async (req, res) => {
			const id = req.params.id;
			const meta = await sources[0]!.extractMeta(
				new URL(`https://www.instagram.com/reel/${id}`),
			);
			return new Response(
				InstagramGraph({
					id,
					title: meta.title,
					originalUrl: meta.url,
				}) as string,
				{
					headers: {
						"Content-Type": "text/html",
					},
				},
			);
		},
	},
});
