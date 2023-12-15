const GetVideoId = require('get-video-id')
const TiktokDL = require("@tobyg74/tiktok-api-dl").TiktokDL

return {
    Name: "Tiktok",
    Match: function(Url) {
        return GetVideoId(Url.href).service == "tiktok"
    },
    ReplaceMessage: async function(Message, Parts) {

    },
    Embed: async function(Url) {
        const RequestData = await TiktokDL(Url.href, { version: "v3" })
        const VideoData = RequestData.result
        console.log(VideoData)
        return {
            Title: VideoData.desc,
            AuthorName: VideoData.author.nickname,
            MediaUrl: VideoData.video2
        }
    }
}