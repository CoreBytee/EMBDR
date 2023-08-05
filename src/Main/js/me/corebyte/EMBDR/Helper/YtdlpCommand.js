const ChildProcess = require("child_process")
const Path = require("path")

const YtdlpPath = Path.join(
    process.cwd,
    "./bin/",
    TypeWriter.OS == "win32" ? "yt-dlp.exe" : "yt-dlp"
)

console.log(YtdlpPath)

module.exports = function(Arguments=[]) {

}