import { env } from "bun";

export const WEBSERVER_PORT = Number.parseInt(env.WEBSERVER_PORT ?? "3000")
export const WEBSERVER_URL = env.WEBSERVER_URL
export const DISCORD_TOKEN = env.DISCORD_TOKEN