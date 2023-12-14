console.log("Hello World!")

const FS = require("fs-extra")
const DiscordJs = require("discord.js")

// return async function() {

//     globalThis.LingBot = {
//         Words: Import("me.corebyte.LingBot.Words"),
//         Config: FS.readJSONSync("./Config.json"),
//         Client: new DiscordJs.Client(
//             {
//                 intents: [3276799],
//             }
//         ),
//         REST: new DiscordJs.REST({ version: "10" }),
//         Commands: [
//             Import("me.corebyte.LingBot.Commands.PingCommand"),
//             Import("me.corebyte.LingBot.Commands.RevealCommand"),
//         ],
//         Emojis: {
//             Blocks: {
//                 Blue: ":blue_square:",
//                 Brown: ":brown_square:",
//                 Green: ":green_square:",
//                 Orange: ":orange_square:",
//                 Purple: ":purple_square:",
//                 Red: ":red_square:",
//                 White: ":white_large_square:",
//                 Yellow: ":yellow_square:",
//             }
//         },
//         WordsManager: Import("me.corebyte.LingBot.WordsManager")
//     }
//     LingBot.REST.setToken(LingBot.Config.Token)

//     await LingBot.Client.login(LingBot.Config.Token);
//     await Import("me.corebyte.LingBot.DeployCommands")()
//     LingBot.Channel = await LingBot.Client.channels.fetch(LingBot.Config.ChannelId)
//     LingBot.Guild = LingBot.Channel.guild
//     await LingBot.WordsManager.LoadHandles()   
//     await LingBot.WordsManager.NewWord()


// }


    globalThis.EMBDR = {
        Config: FS.readJSONSync("./Config.json"),
        Sources: await Import("me.corebyte.EMBDR.Sources.SourceList"),
        Client: new DiscordJs.Client(
            {
                intents: [3276799],
            }
        ),
        DiscordRest: new DiscordJs.REST({ version: "10" }),

        Interactions: await Import("me.corebyte.EMBDR.Interactions.Main")
    }

    EMBDR.DiscordRest.setToken(EMBDR.Config.Token)
    await EMBDR.Client.login(EMBDR.Config.Token)

    await (await Import("me.corebyte.EMBDR.Helper.DeployInteractions"))()
