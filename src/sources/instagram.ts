import type { Message } from "discord.js";
import { WEBSERVER_URL } from "../env";
import useProxy from "../util/useProxy";
import { instagramGetUrl } from "../util/instagramGetUrl";
import random from "just-random";
import database from "../database";

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
	console.log(`Fetching Instagram post metadata for ${id}`);

	// Try to get from cache
	const cached = database
		.query(
			"SELECT id, title, author, video_url FROM instagram_reels WHERE id = ?",
		)
		.get(id) as
		| { id: string; title: string; author: string; video_url: string }
		| undefined;
	if (cached) {
		console.log(`Found Instagram post metadata for ${id} in cache`);

		return {
			id: cached.id,
			title: cached.title,
			url: `https://www.instagram.com/p/${cached.id}`,
			videoUrl: cached.video_url,
		};
	}

	// Not in cache, fetch and cache
	const url = `https://instagram.com/p/${id}`;
	const data = await instagramGetUrl(url, {
		fetchFn: fetchProxy as typeof fetch,
	});
	console.log(`Fetched Instagram post metadata for ${id}`);

	const meta = {
		id: id,
		title: `Post by @${data.post_info.owner_username}`,
		url: `https://www.instagram.com/p/${id}`,
		videoUrl: data.media_details[0]!.url,
	};

	database.run(
		"INSERT OR REPLACE INTO instagram_reels (id, title, author, video_url) VALUES (?, ?, ?, ?)",
		[id, meta.title, data.post_info.owner_username, meta.videoUrl],
	);

	return meta;
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