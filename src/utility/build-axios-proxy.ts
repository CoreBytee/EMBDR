export default function buildAxiosProxy(rawUrl?: string) {
	if (!rawUrl) return undefined;
	const url = new URL(rawUrl);
	return {
		protocol: url.protocol === "http:" ? "http" : "https",
		host: url.hostname,
		port: url.port
			? Number.parseInt(url.port, 10)
			: url.protocol === "http:"
				? 80
				: 443,
		auth: url.username
			? {
					username: url.username,
					password: url.password,
				}
			: undefined,
	};
}
