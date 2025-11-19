import { env } from "bun";
import type { MediaInformation } from "../classes/sources/BaseSource";

export default function Embed(data: MediaInformation) {
	return (
		<html lang="en">
			<head>
				<title>EMBDR</title>

				{/*Type*/}
				<meta property="og:type" content="website" />
				<meta name="twitter:card" content="summary_large_image" />

				{/*Title*/}
				<meta property="og:title" content={data.title} />
				<meta property="twitter:title" content={data.title} />

				{/*Author*/}
				<meta property="twitter:creator" content={data.author} />

				{/*Description*/}
				<meta property="og:description" content={data.description} />

				{/*Site name*/}
				<meta
					property="og:site_name"
					content={env.WEBSERVER_URL as string}
				/>
				<meta
					property="twitter:site"
					content={env.WEBSERVER_URL as string}
				/>

				{/*URL*/}
				<meta property="og:url" content={data.url} />

				{/*Theme color*/}
				<meta property="theme-color" content="#5665f5" />

				{data.media.map((media, index) => {
					switch (media.type) {
						case "video":
							return (
								<>
									{/*Video*/}
									<meta
										property="og:video"
										content={media.url}
									/>
									<meta
										property="twitter:player"
										content={`/m/${data.source}/${data.id}/${index}`}
									/>

									{/*Secure Video*/}
									<meta
										property="og:video:secure_url"
										content={`/m/${data.source}/${data.id}/${index}`}
									/>

									{/*Width*/}
									<meta
										property="og:video:width"
										content={media.dimensions.width.toString()}
									/>
									<meta
										property="twitter:player:width"
										content={media.dimensions.width.toString()}
									/>

									{/*Height*/}
									<meta
										property="og:video:height"
										content={media.dimensions.height.toString()}
									/>
									<meta
										property="twitter:player:height"
										content={media.dimensions.height.toString()}
									/>
								</>
							);

						case "image":
							return (
								<>
									{/*Image*/}
									<meta
										property="og:image"
										content={`/m/${data.source}/${data.id}/${index}`}
									/>
									<meta
										property="twitter:image"
										content={`/m/${data.source}/${data.id}/${index}`}
									/>

									{/*Width*/}
									<meta
										property="og:image:width"
										content={media.dimensions.width.toString()}
									/>

									{/*Height*/}
									<meta
										property="og:image:height"
										content={media.dimensions.height.toString()}
									/>
								</>
							);

						default:
							return null;
					}
				})}

				{/* Redirect immediately */}
				<meta http-equiv="refresh" content={`0; url=${data.url}`} />
			</head>
			<body>
				<a href={data.url}>You are being redirected...</a>
			</body>
		</html>
	);
}
