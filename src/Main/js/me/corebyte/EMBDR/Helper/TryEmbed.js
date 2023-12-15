// const ListLinks = await Import("me.corebyte.EMBDR.Helper.ListLinks")
// const MatchSource = await Import("me.corebyte.EMBDR.Helper.MatchSource")

const DiscordJs = require("discord.js")

const ExtractUrls = await Import("me.corebyte.EMBDR.Helper.ExtractUrls")
const UltraHook = await Import("me.corebyte.UltraHook")

return async function (Options) {
    const Content = Options.Content
    const Channel = Options.Channel
    let Message = Options.Message

    const Urls = ExtractUrls(Content)

    const Parts = Urls.GetParts()

    if (Message) {
        for (const Source of EMBDR.Sources) {
            await Source.ReplaceParts(Parts)
        }

        let NewMessage = ""
        for (const Part of Parts) {
            NewMessage += (Part.Replace || Part.Content) + " "
        }

        if (NewMessage != Content) {
            Message = await UltraHook.ReplaceMessage(Message, NewMessage)
        }
    }

    for (const Index in Parts) {
        const Part = Parts[Index]
        if (!Part.Url) { continue }

        for (const EmbedSource of EMBDR.Sources) {
            const Matches = EmbedSource.Match(Part.Url)
            if (!Matches) { continue }
            const EmbedData = await EmbedSource.Embed(Part.Url)
            const MessageData = `[Embed for ${EmbedData.Title} by ${EmbedData.AuthorName}](${EmbedData.MediaUrl})`
            if (Message) {
                Message.reply(MessageData)
            } else {
                await Channel.send(
                    MessageData
                )
            }
        }
    }
}