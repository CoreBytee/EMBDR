const ListLinks = Import("me.corebyte.EMBDR.Helper.ListLinks")

module.exports = function(Message, Interaction) {
    console.log(Message)

    const MessageContent = Message.content
    const MessageLinks = ListLinks(MessageContent)

    console.log(MessageLinks)

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

    
}