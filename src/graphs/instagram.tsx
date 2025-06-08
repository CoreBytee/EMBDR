export default function InstagramGraph({
	id,
	title,
	originalUrl,
}: {
	id: string;
	title: string;
	originalUrl: string;
}) {
	return (
		<html lang="en">
			<head>
				<title>Document</title>

				{/* Meta tags */}
				<meta charset="UTF-8" />
				<meta name="viewport" content="width=device-width, initial-scale=1.0" />

				{/* Open graph tags */}
				<meta property="og:title" content={title} />
				<meta property="og:video" content={`/v/ig/${id}`} />

				{/* Redirect tag */}
				<meta http-equiv="refresh" content={`0; url=${originalUrl}`} />
			</head>
			<body>
				<a href={originalUrl}>You are being redirected...</a>
			</body>
		</html>
	);
}
