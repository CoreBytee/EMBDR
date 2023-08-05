const LinkFinder = require("links-finder").findLinks

module.exports = function(LinksString) {
    const LinkLocations = LinkFinder(LinksString)
    const Links = LinkLocations.map(LinkLocation => { return LinksString.substring(LinkLocation.start, LinkLocation.end) })
    return Links
}