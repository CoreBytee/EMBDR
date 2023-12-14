const ListLinks = await Import("me.corebyte.EMBDR.Helper.ListLinks")
const MatchSource = await Import("me.corebyte.EMBDR.Helper.MatchSource")

return function(Message, Interaction) {
    const MessageContent = Message.content
    const MessageLinks = ListLinks(MessageContent)

    if (MessageLinks.length == 0) {
        if (Interaction) {
            Interaction.reply(
                {
                    content: "No links found in message. You dummy.",
                    ephemeral: true,
                    tts: true
                }
            )
        }
        return
    }

    const ProcessedLinks = []

    for (const Link of MessageLinks) {
        const SourceData = MatchSource(Link)

        if (!SourceData) { continue }
        if (SourceData.PreProcess) {
            const PreProcessedLink = SourceData.PreProcess(Link)
            if (PreProcessedLink) {
                ProcessedLinks.push(PreProcessedLink)
            }
        }
    }

    const MissingCount = MessageLinks.length - ProcessedLinks.length
    if (MissingCount > 0 && Interaction) {
        Interaction.reply(
            {
                content: `Some links (${MissingCount}) did not have any connected extractors and will be missing.`,
                ephemeral: true
            }
        )
    }

    const ExtractionMessages = []

    for (const Link of ProcessedLinks) {
        Message.channel.send(
            {
                embeds: [
                    
                ]
            }
        )
    }



    
}