const DiscordJs = require("discord.js")
const ContextMenuCommandBuilder = DiscordJs.ContextMenuCommandBuilder
const ApplicationCommandType = DiscordJs.ApplicationCommandType

const TryEmbed = await Import("me.corebyte.EMBDR.Helper.TryEmbed")

return {
    Type: "MessageContext",
    Name: "Embed Links",

    Data: new ContextMenuCommandBuilder()
        .setName("Embed Links")
        .setType(ApplicationCommandType.Message),

    Handler: async function(Interaction) {
        const Message = Interaction.targetMessage
        return await TryEmbed(Message, Interaction)
    }
}