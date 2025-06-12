import type { Message } from "discord.js";
import { WEBSERVER_URL } from "../env";
import useProxy from "../util/useProxy";
import { instagramGetUrl } from "../util/instagramGetUrl";
import random from "just-random";

const fetchProxy = await useProxy();

const messages = [
	"Beep beep, your embed is ready!",
	"Embed? Op je muil gauw!",
	"Here's your spicy Instagram drop!",
	"Fresh from the 'Gram, just for you!",
	"Look what I found on Insta!",
	"Your daily dose of Instagram, served hot!",
	"Instagram magic, coming right up!",
	"Another banger from the 'Gram!",
	"Sliding into your DMs with this embed!",
	"Insta-nt delivery, enjoy your post!",
	"Want to see dead people? Check this out!",
];

export const name = "Instagram";
export const short = "ig";

export async function matchUrl(url: URL) {
	return (
		url.hostname === "www.instagram.com" ||
		url.hostname === "instagram.com" ||
		url.hostname === "ddinstagram.com"
	);
}

export async function extractId(url: URL) {
	const path = url.pathname.split("/");
	const id = path[path.length - 1] || path[path.length - 2];
	return id;
}

export async function extractMeta(id: string) {
	const url = `https://instagram.com/p/${id}`;
	console.log(`Fetching Instagram post metadata for ${url}`);
	const data = await instagramGetUrl(url, {
		fetchFn: fetchProxy as typeof fetch,
	});
	console.log(`Fetched Instagram post metadata for ${url}`);

	return {
		id: id,
		title: `Post by @${data.post_info.owner_username}`,
		url: `https://www.instagram.com/p/${id}`,
		thumbnailUrL: data.media_details[0]!.thumbnail!,
		videoUrl: data.media_details[0]!.url,
	};
}

export async function reply(url: URL, message: Message) {
	url.hostname = "instagram.com";
	url.search = "";

	const id = await extractId(url);
	const funny = random(messages);

	message.reply({
		content: `[${funny}](${WEBSERVER_URL}/e/ig/${id})`,
		embeds: [],
		allowedMentions: { repliedUser: false, users: [], roles: [] },
	});
}