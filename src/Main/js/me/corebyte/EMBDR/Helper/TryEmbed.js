// const ListLinks = await Import("me.corebyte.EMBDR.Helper.ListLinks")
// const MatchSource = await Import("me.corebyte.EMBDR.Helper.MatchSource")

const DiscordJs = require("discord.js")

const ExtractUrls = await Import("me.corebyte.EMBDR.Helper.ExtractUrls")

return async function(Options) {
    const Content = Options.Content
    const Channel = Options.Channel
    const Message = Options.Message

    const Urls = ExtractUrls(Content)

    const Parts = Urls.GetParts()
    for (const Index in Parts) {
        const Part = Parts[Index]
        if (!Part.Url) { continue }

        for (const EmbedSource of EMBDR.Sources) {
            const Matches = EmbedSource.Match(Part.Url)
            if (!Matches) { continue }
            const EmbedData = await EmbedSource.Embed(Part.Url)
            console.log(EmbedData)
            const Embed = new DiscordJs.EmbedBuilder()
            Embed.setTitle(EmbedData.Title)
            Embed.setAuthor({name: EmbedData.AuthorName})
            Embed.setImage(EmbedData.MediaUrl)
            await Channel.send(
                {
                    content: `[](${EmbedData.MediaUrl})`,
                    // embeds: [Embed]
                }
            )
        }
    }
}