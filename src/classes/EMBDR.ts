import DiscordBot from "./DiscordBot";
import type BaseSource from "./sources/BaseSource";
import InstagramSource from "./sources/InstagramSource";
import WebServer from "./WebServer";

export default class EMBDR {
	sources: BaseSource[];
	discord: DiscordBot;
	webServer: WebServer;
	constructor() {
		console.info("[embdr] starting...");
		this.sources = [new InstagramSource()];

		this.discord = new DiscordBot(this.sources);
		this.webServer = new WebServer(this.sources);
	}
}
