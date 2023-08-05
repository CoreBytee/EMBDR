const DiscordJs = require("discord.js")

module.exports = async function() {
    TypeWriter.Logger.Information("Deploying interactions...")

    EMBDR.DiscordRest.put(
        DiscordJs.Routes.applicationGuildCommands(EMBDR.Client.user.id, EMBDR.Config.GuildId),
        { body: EMBDR.Interactions.map(Interaction => Interaction.Data.toJSON()) }
    )

    TypeWriter.Logger.Information("Deployed interactions...")

    EMBDR.Client.on(
        "interactionCreate",
        async function(ReceivedInteraction) {

            var Type
            if (ReceivedInteraction.isMessageContextMenuCommand()) {
                Type = "MessageContext"
            }

            TypeWriter.Logger.Information(`Received ${Type} interaction...`)

            var FoundInteraction = EMBDR.Interactions.find(
                function(Interaction) {
                    return Interaction.Type == Type && Interaction.Name == ReceivedInteraction.commandName
                }
            )

            if (!FoundInteraction || !FoundInteraction.Handler) {
                TypeWriter.Logger.Error(`Handler for ${Type} interaction not found name ${ReceivedInteraction.commandName}`)
                return
            }

            return await FoundInteraction.Handler(ReceivedInteraction)
        }
    )

}