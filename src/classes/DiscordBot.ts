import { env } from "bun";
import { Client, MessageFlags } from "discord.js";
import getUrls from "get-urls";
import random from "just-random";
import type BaseSource from "./sources/BaseSource";

export default class DiscordBot {
	client: Client<boolean>;
	constructor(sources: BaseSource[]) {
		this.client = new Client({ intents: [53608447] });

		console.info("[discord] logging in to discord");
		this.client.login(env.DISCORD_TOKEN as string);
		this.client.on("clientReady", () => {
			console.info("[discord] connected to discord");
		});

		this.client.on("messageCreate", async (message) => {
			if (message.author.bot) return;
			if (!message.inGuild()) return;

			const content = message.content;
			const urls = getUrls(content);

			for (const urlString of urls) {
				const url = new URL(urlString);
				for (const source of sources) {
					const match = await source.match(url);
					if (!match) continue;

					const id = await source.extractId(url);
					if (!id) continue;

					console.info(`[discord] embedding ${id} from ${source.name}`);
					const response = random(source.messages);
					message.suppressEmbeds(true);
					message.reply({
						content: `[${response}](${env.WEBSERVER_URL}/e/${source.short}/${id})`,
						flags: [MessageFlags.SuppressNotifications],
						allowedMentions: {
							repliedUser: false,
							users: [],
							roles: [],
						},
					});
					return;
				}
			}
		});
	}
}
