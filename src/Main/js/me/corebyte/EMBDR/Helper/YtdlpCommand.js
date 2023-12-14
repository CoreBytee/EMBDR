const ChildProcess = require("child_process")
const Path = require("path")

const YtdlpPath = Path.join(
    process.cwd,
    "./Binaries/",
    TypeWriter.OS == "win32" ? "yt-dlp.exe" : "yt-dlp"
)

console.log(YtdlpPath)

return function(Arguments=[]) {
    console.log(ChildProcess.spawnSync(YtdlpPath))
}