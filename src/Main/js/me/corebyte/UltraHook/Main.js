const DiscordJs = require("discord.js")

class UltraHook {

    static async ReplaceMessage(Message, Content) {
        await Message.delete()
        return await this.CloneUser(
            Message.channel,
            Message.author,
            Content
        )
    }

    static async CloneMessage(Message) {
        return await this.CloneUser(
            Message.channel,
            Message.author,
            Message.content
        )
    }

    static async CloneUser(Channel, User, Content) {
        return await this.Send(
            Channel,
            Content,
            User.globalName,
            User.avatarURL()
        )
    }

    static async Send(Channel, Content, Username, AvatarUrl) {
        if (typeof Content == "string") {
            Content = {
                content: Content
            }
        }

        Content.username = Username
        Content.avatarURL = AvatarUrl

        const Webhook = await this.GetUltraHook(Channel)
        return await Webhook.send(Content)
    }

    static async GetUltraHook(Channel) {
        const Webhooks = await Channel.fetchWebhooks()
        const UltraHook = Webhooks.find(
            Webhook => Webhook.name == "UltraHook"
        )

        if (UltraHook) { return UltraHook }

        return await Channel.createWebhook(
            {
                name: "UltraHook",
                reason: "Missing UltraHook"
            }
        )
    }
}

return UltraHook