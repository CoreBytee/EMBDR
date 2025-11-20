import { Database } from "bun:sqlite";
import { ensureDirSync } from "fs-extra";
import {
	instagramGetId,
	instagramGetRedirect,
	instagramGetUrl,
} from "instagram-url-direct";
import BaseSource, { type MediaInformation } from "./BaseSource";

ensureDirSync("./cache");
const cache = new Database("./cache/instagram.db");

cache.run(`
	CREATE TABLE IF NOT EXISTS "posts" (
		id TEXT PRIMARY KEY,
		data TEXT NOT NULL
	)
`);

export default class InstagramSource extends BaseSource {
	name = "Instagram";
	short = "ig";
	messages = [
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

	async match(url: URL) {
		return (
			url.hostname === "www.instagram.com" ||
			url.hostname === "instagram.com" ||
			url.hostname === "ddinstagram.com"
		);
	}

	async extractId(url: URL) {
		try {
			const redirected = await instagramGetRedirect(url.toString());
			return await instagramGetId(redirected);
		} catch {
			return null;
		}
	}

	async extractMedia(id: string) {
		const cached = await this.readCache(id);
		if (cached) {
			console.info(`[instagram] got post ${id} from cache`);
			return cached;
		}

		const data = await this.requestPost(id);
		if (!data) return null;

		cache
			.prepare(
				`
				INSERT OR REPLACE INTO "posts" ("id", "data") VALUES (?, ?)
			`,
			)
			.run(id, JSON.stringify(data));

		console.info(`[instagram] got post ${id} from the api`);

		return data;
	}

	private async readCache(id: string) {
		const row = await cache
			.query(`SELECT "data" from "posts" where "id" = ?`)
			.get(id);

		const data = row as { data: string } | undefined;
		if (!data) return null;
		return JSON.parse(data.data);
	}

	private async requestPost(id: string): Promise<MediaInformation | null> {
		try {
			const data = await instagramGetUrl(id);
			return {
				id: id,
				source: this.short,
				title: `Post by ${data.post_info.owner_fullname}`,
				description: data.post_info.caption,
				author: data.post_info.owner_username,
				url: `https://www.instagram.com/p/${id}`,
				media: data.media_details.map((media) => ({
					type: media.type as "video" | "image",
					url: media.url,
					dimensions: media.dimensions,
				})),
			};
		} catch (e) {
			console.info(`[instagram] requesting post ${id} failed: ${e}`);
			return null;
		}
	}
}
