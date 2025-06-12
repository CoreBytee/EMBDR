import createFetch from "@corebyte/proxy";

const PROXY_ENABLED =
	process.env.PROXY_ENABLED === "true" || process.env.PROXY_ENABLED === "1";
const PROXY_SSL =
	process.env.PROXY_SSL === "true" || process.env.PROXY_SSL === "1";
const PROXY_HOST = process.env.PROXY_HOST;
const PROXY_PORT = process.env.PROXY_PORT;
const PROXY_SECRET = process.env.PROXY_SECRET;

let proxy: ReturnType<typeof createFetch> | null = null;
export default async function useProxy() {
	if (!PROXY_ENABLED) return fetch;
	if (!PROXY_HOST?.trim())
		throw new Error("Proxy host is unset. Please check your .env file.");
	if (!PROXY_PORT?.trim())
		throw new Error("Proxy port is unset. Please check your .env file.");
	if (!PROXY_SECRET?.trim())
		throw new Error("Proxy secret is unset. Please check your .env file.");

	if (proxy) return proxy;

	if (!PROXY_PORT?.trim() || !PROXY_SECRET?.trim()) {
		throw new Error(
			"Proxy port or secret is unset. Please check your .env file.",
		);
	}

	proxy = createFetch({
		ssl: PROXY_SSL,
		host: PROXY_HOST.trim(),
		port: Number.parseInt(PROXY_PORT),
		secret: PROXY_SECRET.trim(),
	});

	return proxy;
}
