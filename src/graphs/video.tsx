export default function VideoGraph({
	title,
	url,
	video,
}: {
	title: string;
	url: string;
	video: string;
}) {
	return (
		<html lang="en">
			<head>
				<title>EMBDR</title>

				{/* Meta tags */}
				<meta charset="UTF-8" />
				<meta name="viewport" content="width=device-width, initial-scale=1.0" />

				{/* Open graph tags */}
				<meta property="og:title" content={title} />
				<meta property="og:url" content={"https://example.com"} />

				<meta property="og:video:url" content={video} />
				<meta property="og:video:secure_url" content={video} />
				<meta property="og:video:type" content="video/mp4" />

				<meta property="og:type" content="video" />

				{/* Redirect tag */}
				<meta http-equiv="refresh" content={`0; url=${url}`} />
			</head>
			<body>
				<a href={url}>You are being redirected...</a>
			</body>
		</html>
	);
}
