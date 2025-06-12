import qs from "qs";

//Interface
export interface InstagramResponse {
	results_number: number;
	url_list: string[];
	post_info: {
		owner_username: string;
		owner_fullname: string;
		is_verified: boolean;
		is_private: boolean;
		likes: number;
		is_ad: boolean;
		caption: string;
	};
	media_details: {
		type: string;
		dimensions: {
			height: number;
			width: number;
		};
		url: string;
		video_view_count?: number;
		thumbnail?: string;
	}[];
}

export interface InstagramError {
	error: string;
}

//Main function
export async function instagramGetUrl(
	url_media: string,
	config: { retries?: number; delay?: number; fetchFn?: typeof fetch } = {
		retries: 5,
		delay: 1000,
	},
): Promise<InstagramResponse> {
	const fetchFn = config.fetchFn || fetch;
	return new Promise<InstagramResponse>((resolve, reject) => {
		(async () => {
			try {
				const checkedUrl = await checkRedirect(url_media, fetchFn);
				const SHORTCODE = getShortcode(checkedUrl);
				if (!SHORTCODE)
					throw new Error("Shortcode could not be determined from URL");
				const INSTAGRAM_REQUEST = await instagramRequest(
					SHORTCODE,
					config.retries ?? 5,
					config.delay ?? 1000,
					fetchFn,
				);
				const OUTPUT_DATA = createOutputData(INSTAGRAM_REQUEST);
				resolve(OUTPUT_DATA as InstagramResponse);
			} catch (err) {
				reject(err);
			}
		})();
	});
}

//Utilities
async function checkRedirect(
	url: string,
	fetchFn: typeof fetch,
): Promise<string> {
	const split_url = url.split("/");

	if (split_url.includes("share")) {
		const res = await fetchFn(url, { redirect: "follow" });
		if (!res.ok) throw new Error(`Failed to fetch redirect: ${res.status}`);
		return res.url;
	}

	return url;
}

interface PostInfo {
	owner_username: string;
	owner_fullname: string;
	is_verified: boolean;
	is_private: boolean;
	likes: number;
	is_ad: boolean;
	caption: string;
}

interface MediaDetails {
	type: string;
	dimensions: { height: number; width: number };
	url: string;
	video_view_count?: number;
	thumbnail?: string;
}

// --- Instagram GraphQL Types ---
interface InstagramOwner {
	username: string;
	full_name: string;
	is_verified: boolean;
	is_private: boolean;
}

interface InstagramCaptionEdge {
	node: { text: string };
}

interface InstagramCaption {
	edges: InstagramCaptionEdge[];
}

interface InstagramLike {
	count: number;
}

interface InstagramMediaBase {
	__typename: string;
	is_ad: boolean;
	owner: InstagramOwner;
	edge_media_to_caption: InstagramCaption;
	edge_media_preview_like: InstagramLike;
	display_url: string;
	dimensions: { height: number; width: number };
	is_video: boolean;
}

interface InstagramVideoMedia extends InstagramMediaBase {
	is_video: true;
	video_url: string;
	video_view_count: number;
}

interface InstagramImageMedia extends InstagramMediaBase {
	is_video: false;
}

interface InstagramSidecarChild {
	node: InstagramVideoMedia | InstagramImageMedia;
}

interface InstagramSidecar {
	__typename: "XDTGraphSidecar";
	edge_sidecar_to_children: { edges: InstagramSidecarChild[] };
	// plus all InstagramMediaBase fields
	is_ad: boolean;
	owner: InstagramOwner;
	edge_media_to_caption: InstagramCaption;
	edge_media_preview_like: InstagramLike;
	display_url: string;
	dimensions: { height: number; width: number };
	is_video: boolean;
}

type InstagramGraphQLMedia =
	| InstagramVideoMedia
	| InstagramImageMedia
	| InstagramSidecar;

function formatPostInfo(requestData: InstagramGraphQLMedia): PostInfo {
	try {
		const mediaCapt = requestData.edge_media_to_caption?.edges;
		let capt = "";
		if (
			Array.isArray(mediaCapt) &&
			mediaCapt.length > 0 &&
			mediaCapt[0] &&
			mediaCapt[0].node &&
			typeof mediaCapt[0].node.text === "string"
		) {
			capt = mediaCapt[0].node.text;
		}
		return {
			owner_username: requestData.owner.username,
			owner_fullname: requestData.owner.full_name,
			is_verified: requestData.owner.is_verified,
			is_private: requestData.owner.is_private,
			likes: requestData.edge_media_preview_like.count,
			is_ad: requestData.is_ad,
			caption: capt,
		};
	} catch (err) {
		throw new Error(`Failed to format post info: ${(err as Error).message}`);
	}
}

function formatMediaDetails(
	mediaData: InstagramVideoMedia | InstagramImageMedia,
): MediaDetails {
	try {
		if (mediaData.is_video) {
			return {
				type: "video",
				dimensions: mediaData.dimensions,
				video_view_count: mediaData.video_view_count,
				url: mediaData.video_url,
				thumbnail: mediaData.display_url,
			};
		}
		return {
			type: "image",
			dimensions: mediaData.dimensions,
			url: mediaData.display_url,
		};
	} catch (err) {
		throw new Error(
			`Failed to format media details: ${(err as Error).message}`,
		);
	}
}

function getShortcode(url: string): string | undefined {
	try {
		const split_url = url.split("/");
		const post_tags = ["p", "reel", "tv", "reels"];
		const index_shortcode =
			split_url.findIndex((item) => post_tags.includes(item)) + 1;
		const shortcode = split_url[index_shortcode];
		return shortcode;
	} catch (err) {
		throw new Error(`Failed to obtain shortcode: ${(err as Error).message}`);
	}
}

async function getCSRFToken(fetchFn: typeof fetch): Promise<string> {
	try {
		const res = await fetchFn("https://www.instagram.com/", {
			method: "GET",
			credentials: "include",
		});
		if (!res.ok) throw new Error(`Failed to fetch CSRF token: ${res.status}`);
		const cookies = res.headers.get("set-cookie");
		if (!cookies) throw new Error("CSRF token not found in response headers.");
		const csrfMatch = cookies.match(/csrftoken=([^;]+)/);
		if (!csrfMatch || !csrfMatch[1])
			throw new Error("CSRF token not found in cookies.");
		return csrfMatch[1];
	} catch (err) {
		throw new Error(`Failed to obtain CSRF: ${(err as Error).message}`);
	}
}

function isSidecar(
	requestData: InstagramGraphQLMedia,
): requestData is InstagramSidecar {
	return requestData.__typename === "XDTGraphSidecar";
}

async function instagramRequest(
	shortcode: string,
	retries: number,
	delay: number,
	fetchFn: typeof fetch,
): Promise<InstagramGraphQLMedia> {
	try {
		const BASE_URL = "https://www.instagram.com/graphql/query";
		const INSTAGRAM_DOCUMENT_ID = "9510064595728286";
		const dataBody = qs.stringify({
			variables: JSON.stringify({
				shortcode: shortcode,
				fetch_tagged_user_count: null,
				hoisted_comment_id: null,
				hoisted_reply_id: null,
			}),
			doc_id: INSTAGRAM_DOCUMENT_ID,
		});

		const token = await getCSRFToken(fetchFn);
		const headers: Record<string, string> = {
			"X-CSRFToken": token ?? "",
			"Content-Type": "application/x-www-form-urlencoded",
		};

		const res = await fetchFn(BASE_URL, {
			method: "POST",
			headers,
			body: dataBody,
			credentials: "include",
		});
		if (!res.ok) {
			if ([429, 403].includes(res.status) && retries > 0) {
				const retryAfter = res.headers.get("retry-after");
				const waitTime = retryAfter
					? Number.parseInt(retryAfter) * 1000
					: delay;
				await new Promise((res) => setTimeout(res, waitTime));
				return instagramRequest(shortcode, retries - 1, delay * 2, fetchFn);
			}
			throw new Error(`Failed instagram request: ${res.status}`);
		}
		const data = (await res.json()) as {
			data?: { xdt_shortcode_media?: InstagramGraphQLMedia };
		};
		if (
			!data ||
			typeof data !== "object" ||
			!("data" in data) ||
			!data.data?.xdt_shortcode_media
		) {
			throw new Error(
				"Only posts/reels supported, check if your link is valid.",
			);
		}
		return data.data.xdt_shortcode_media;
	} catch (err) {
		throw new Error(`Failed instagram request: ${(err as Error).message}`);
	}
}

function createOutputData(requestData: InstagramGraphQLMedia) {
	try {
		const url_list: string[] = [];
		const media_details: MediaDetails[] = [];
		if (isSidecar(requestData)) {
			//Post with sidecar
			requestData.edge_sidecar_to_children.edges.forEach((media) => {
				media_details.push(formatMediaDetails(media.node));
				if (media.node.is_video) {
					//Sidecar video item
					url_list.push(media.node.video_url);
				} else {
					//Sidecar image item
					url_list.push(media.node.display_url);
				}
			});
		} else {
			//Post without sidecar
			media_details.push(formatMediaDetails(requestData));
			if (requestData.is_video) {
				// Video media
				url_list.push(requestData.video_url);
			} else {
				//Image media
				url_list.push(requestData.display_url);
			}
		}

		return {
			results_number: url_list.length,
			url_list,
			post_info: formatPostInfo(requestData),
			media_details,
		};
	} catch (err) {
		throw new Error(`Failed to create output data: ${(err as Error).message}`);
	}
}
