const DiscordJs = require("discord.js")

const Logger = TypeWriter.CreateLogger("InteractionDeployer")

return async function(Interactions, Client, Rest, GuildId) {
    Logger.Information("Deploying interactions...")

    Rest.put(
        DiscordJs.Routes.applicationCommands(Client.user.id),
        { body: Interactions.map(Interaction => Interaction.Data.toJSON()) }
    )

    Rest.put(
        DiscordJs.Routes.applicationGuildCommands(Client.user.id, GuildId),
        { body: [] }
    )

    Logger.Information("Deployed interactions...")

    Client.on(
        "interactionCreate",
        async function(ReceivedInteraction) {

            var Type
            if (ReceivedInteraction.isMessageContextMenuCommand()) {
                Type = "MessageContext"
            }

            TypeWriter.Logger.Information(`Received ${Type} interaction...`)

            var FoundInteraction = Interactions.find(
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