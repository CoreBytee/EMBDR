use url::Url;

pub fn extract_urls(string: &String) -> Vec<Url> {
    let mut urls = Vec::new();
    for word in string.split_whitespace() {
        if let Ok(url) = Url::parse(word) {
            urls.push(url);
        }
    }
    urls
}
