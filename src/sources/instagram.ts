import type { Message } from "discord.js";
import { instagramGetUrl } from "instagram-url-direct";
import { WEBSERVER_URL } from "../env";

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
	const data = await instagramGetUrl(url);
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

	message.reply({
		content: `${WEBSERVER_URL}/e/ig/${id}`,
		embeds: [],
		allowedMentions: { repliedUser: false, users: [], roles: [] },
	});
}
