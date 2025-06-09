import { Client } from "discord.js";
import getUrls from "get-urls";
import { DISCORD_TOKEN, WEBSERVER_PORT, WEBSERVER_URL } from "./env";
import VideoGraph from "./graphs/video";
import * as instagramSource from "./sources/instagram";

const sources = [instagramSource];

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
		"/e/ig/:id": async (request, response) => {
			const id = request.params.id;
			const meta = await instagramSource.extractMeta(id);
			return new Response(
				VideoGraph({
					title: meta.title,
					url: meta.url,
					video: `${WEBSERVER_URL}/v/ig/${id}`,
					thumbnail: `${WEBSERVER_URL}/t/ig/${id}`,
				}) as string,
				{
					headers: {
						"Content-Type": "text/html",
					},
				},
			);
		},
		"/t/ig/:id": async (request, response) => {
			const meta = await instagramSource.extractMeta(request.params.id);
			return Response.redirect(meta.thumbnailUrL, 302);
		},
		"/v/ig/:id": async (request, response) => {
			const meta = await instagramSource.extractMeta(request.params.id);
			return Response.redirect(meta.videoUrl, 302);
		},
	},
});
