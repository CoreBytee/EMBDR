import type { Message } from "discord.js";
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

export async function extractMeta(url: URL) {
	const result = Bun.spawnSync({
		cmd: ["./yt-dlp", "-f", "b", "-J", "-q", url.toString()],
	});

	const data = JSON.parse(result.stdout.toString());
	console.log(data);

	return {
		title: data.title,
		url: data.url,
	};
}

export async function reply(url: URL, message: Message) {
	url.hostname = "ddinstagram.com";
	url.search = "";

	const id = await extractId(url);

	message.reply({
		content: `${WEBSERVER_URL}/e/ig/${id}`,
		embeds: [],
		allowedMentions: { repliedUser: false, users: [], roles: [] },
	});
}

// ytdlp.exe -o "download/%(title)s.%(ext)s" "https://www.instagram.com/reel/DGNnglOz0i2/?igsh=azR2YzljenRsaXU2" -f b -J -q -v
