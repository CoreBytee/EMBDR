module.exports = [
    // {
    //     Name: "Youtube",
    //     Domains: [
    //         "www.youtube.com",
    //         "youtube.com",
    //         "youtu.be"
    //     ],

    //     PreProcess: function(Url) {
            
    //     },
    // },

    {
        Name: "Reddit",
        Domains: [
            "www.reddit.com",
            "reddit.com"
        ],
        PreProcess: function(Url) {
            return Url
        },
        IsBad: function(Url) {
            return false
        }
    }
]