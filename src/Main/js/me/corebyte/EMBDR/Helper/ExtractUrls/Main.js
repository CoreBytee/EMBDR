function IsUrl (Part) {
    let Url
    try {
        Url = new URL(Part)        
    } catch (error) { return false }
    const IsHttp = Url.protocol === "http:" || Url.protocol === "https:"
    if (!IsHttp) { return false }
    return Url
}

class UrlSentence {
    constructor(Sentence) {
        this.Sentence = Sentence
        this.Parts = []
        
        for (const Part of Sentence.split(" ")) {
            console.log(Part)
            this.Parts.push(
                {
                    Content: Part,
                    Url: IsUrl(Part)
                }
            )
        }
    }

    GetParts() {
        return this.Parts
    }
}

function ExtractUrls(Sentence) {
    return new UrlSentence(Sentence)
}

return ExtractUrls