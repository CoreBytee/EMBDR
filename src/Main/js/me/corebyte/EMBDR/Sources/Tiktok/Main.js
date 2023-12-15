const GetVideoId = require('get-video-id')
const TiktokDL = require("@tobyg74/tiktok-api-dl").TiktokDL
const Fetch = require("node-fetch")

const Replacements = [
    "🤧",
    "🤮",
    "🤕",
    "🤒",
    "🤢",
    "🥴",
    "🤐",
    "😮‍💨",
    "😬",
    "🙄",
    "😯",
    "🙈"
]

function RandomReplacement() {
    return Replacements[Math.floor(Math.random() * Replacements.length)]
}

async function GetMediaUrl(Url) {
    const Response = await Fetch(
        "https://api.quickvids.win/v1/shorturl/create",
        {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify(
                {
                    "input_text": Url
                }
            )
        }
    )

    const Data = await Response.json()
    return Data.quickvids_url
}

return {
    Name: "Tiktok",
    Match: function(Url) {
        return GetVideoId(Url.href).service == "tiktok"
    },
    ReplaceParts: async function(Parts) {
        for (const Part of Parts) {
            if (!Part.Url) { continue }
            const Service = GetVideoId(Part.Url.href).service
            if (Service != "tiktok") { continue }
            Part.Replace = RandomReplacement()
        }
    },
    Embed: async function(Url) {
        const [VideoRequest, MediaData] = await Promise.all(
            [
                TiktokDL(Url.href, { version: "v3" }),
                GetMediaUrl(Url.href)
            ]
        )
        const VideoData = VideoRequest.result
        return {
            Title: VideoData.desc,
            AuthorName: VideoData.author.nickname,
            MediaUrl: MediaData
        }
    }
}