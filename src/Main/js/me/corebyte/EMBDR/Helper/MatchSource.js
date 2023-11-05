const SourceList = Import("me.corebyte.EMBDR.Sources.SourceList")

module.exports = function(Url) {
    TypeWriter.Logger.Information(`Matching source for ${Url}`)

    const UrlData = new URL(Url)
    var MatchedSource
    for (const Source of SourceList) {
        const SourceMatchesUrl = Source.Domains.includes(UrlData.hostname)
        if (SourceMatchesUrl) {
            MatchedSource = Source
            break
        }
    }

    if (!MatchedSource) { return false }
    return MatchedSource
}