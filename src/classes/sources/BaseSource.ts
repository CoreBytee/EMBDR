type MaybePromise<T> = T | Promise<T>;

export type MediaInformation = {
	id: string;
	source: string;
	title: string;
	description: string;
	author: string;
	url: string;
	media: {
		url: string;
		type: "video" | "image";
		dimensions: {
			width: number;
			height: number;
		};
	}[];
};

export default abstract class BaseSource {
	abstract readonly name: string;
	abstract readonly short: string;
	abstract readonly messages: string[];

	/**
	 *
	 * @param url URL to match
	 * @returns if the url matched this source
	 */
	abstract match(url: URL): MaybePromise<boolean>;

	/**
	 *
	 * @param url URL to extract id from
	 * @returns extracted id or null if not found
	 */
	abstract extractId(url: URL): MaybePromise<string | null>;

	/**
	 *
	 * @param id id to extract data from
	 * @returns MediaInformation object
	 */
	abstract extractMedia(id: string): MaybePromise<MediaInformation | null>;
}
