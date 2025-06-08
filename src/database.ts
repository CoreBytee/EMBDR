import { Database } from "bun:sqlite";

const database = new Database("cache.db");
database.run(`CREATE TABLE IF NOT EXISTS "instagram_reels" (
	id TEXT PRIMARY KEY,
	title TEXT NOT NULL,
	author TEXT NOT NULL,
	video_url TEXT NOT NULL
)`);

export default database;
