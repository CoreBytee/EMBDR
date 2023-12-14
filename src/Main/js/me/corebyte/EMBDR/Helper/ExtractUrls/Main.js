function IsUrl (Part) {
    const Url = new URL(Part)
    console.log(Url)
    const IsHttp = Url.protocol === "http:" || Url.protocol === "https:"
}

class UrlSentence {
    constructor(Sentence) {
        this.Sentence = Sentence
        this.Parts = []
        
        for (const Part of Sentence) {
            
        }
    }
}

function ExtractUrls(Sentence) {
    return new UrlSentence(Sentence)
}



return ExtractUrls