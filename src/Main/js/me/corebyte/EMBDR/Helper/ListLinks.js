const LinkFinder = require("extract-urls")

return function(Str) {
    return LinkFinder(Str)
}