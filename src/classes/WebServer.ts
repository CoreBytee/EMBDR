import { env } from "bun";
import Embed from "../pages/embed";
import type BaseSource from "./sources/BaseSource";

export default class WebServer {
	constructor(sources: BaseSource[]) {
		const host = env.WEBSERVER_URL as string;
		const port = env.WEBSERVER_PORT as string;

		console.info(`[webserver] listening on ${port} at ${host}`);
		Bun.serve({
			idleTimeout: 30,
			port: env.WEBSERVER_PORT as unknown as number,
			routes: {
				"/e/:sourceId/:mediaId": async (request) => {
					const { sourceId, mediaId } = request.params;

					const source = sources.find((s) => s.short === sourceId);
					if (!source)
						return new Response(
							`The source with id "${sourceId}" is unknown.`,
							{ status: 404 },
						);

					const media = await source.extractMedia(mediaId);
					if (!media)
						return new Response(
							`The media with id "${mediaId}" is unknown for source "${sourceId}".`,
							{ status: 404 },
						);

					return new Response(Embed(media) as string, {
						headers: {
							"Content-Type": "text/html",
						},
					});
				},

				"/m/:sourceId/:mediaId/:mediaIndex": async (request) => {
					const { sourceId, mediaId, mediaIndex } = request.params;

					const source = sources.find((s) => s.short === sourceId);
					if (!source)
						return new Response(
							`The source with id "${sourceId}" is unknown.`,
							{ status: 404 },
						);

					const media = await source.extractMedia(mediaId);
					if (!media)
						return new Response(
							`The media with id "${mediaId}" is unknown for source "${sourceId}".`,
							{ status: 404 },
						);

					const index = parseInt(mediaIndex, 10);
					if (
						Number.isNaN(index) ||
						index < 0 ||
						index >= media.media.length
					)
						return new Response(
							`The media index "${mediaIndex}" is invalid for media "${mediaId}" on source "${sourceId}".`,
							{ status: 404 },
						);

					const selectedMedia = media.media[index]!;

					return new Response(null, {
						status: 302,
						headers: {
							Location: selectedMedia.url,
						},
					});
				},
			},
		});
	}
}
