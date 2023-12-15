const UltraHook = await Import("me.corebyte.UltraHook.Main")
const TryEmbed = await Import("me.corebyte.EMBDR.Helper.TryEmbed")

EMBDR.DiscordJs.Client.on(
    "messageCreate",
    async (Message) => {
        if (Message.author.bot) { return }
        console.log(Message)

        await TryEmbed(
            {
                Content: Message.content,
                Channel: Message.channel,
                Message: Message
            }
        )

    }
)