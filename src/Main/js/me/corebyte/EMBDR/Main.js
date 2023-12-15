console.log("Hello World!")

const FS = require("fs-extra")
const DiscordJs = require("discord.js")

const DeployInteractions = await Import("me.corebyte.DeployInteractions")
const Interactions = await Import("me.corebyte.EMBDR.Interactions")

globalThis.EMBDR = {}
EMBDR.Config = FS.readJSONSync("./Config.json")
EMBDR.Sources = await Import("me.corebyte.EMBDR.Sources")
EMBDR.DiscordJs = {}
EMBDR.DiscordJs.Rest = new DiscordJs.REST({ version: "10" })
EMBDR.DiscordJs.Client = new DiscordJs.Client(
    {
        intents: [3276799],
    }
)

EMBDR.DiscordJs.Rest.setToken(EMBDR.Config.Token)
await EMBDR.DiscordJs.Client.login(EMBDR.Config.Token)
DeployInteractions(Interactions, EMBDR.DiscordJs.Client, EMBDR.DiscordJs.Rest)

await Import("me.corebyte.EMBDR.Events.MessageCreate")