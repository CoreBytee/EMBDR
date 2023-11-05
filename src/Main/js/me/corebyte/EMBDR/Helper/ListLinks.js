const LinkFinder = require("extract-urls")

module.exports = function(Str) {
    return LinkFinder(Str)
}