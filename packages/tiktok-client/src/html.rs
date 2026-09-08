use scraper::Html;

pub fn html_to_markdown(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut result = String::new();
    let root = document.root_element();
    render_node(&root, &mut result);
    result
}

fn render_node(node: &scraper::ElementRef, output: &mut String) {
    for child in node.children() {
        match child.value() {
            scraper::Node::Text(text) => {
                output.push_str(text);
            }
            scraper::Node::Element(element) => {
                match element.name() {
                    "b" => {
                        output.push_str("**");
                        if let Some(element_ref) = scraper::ElementRef::wrap(child) {
                            render_node(&element_ref, output);
                        }
                        output.push_str("**");
                    }
                    "br" => {
                        output.push('\n');
                    }
                    "a" => {
                        if let Some(element_ref) = scraper::ElementRef::wrap(child) {
                            render_node(&element_ref, output);
                        }
                    }
                    _ => {
                        if let Some(element_ref) = scraper::ElementRef::wrap(child) {
                            render_node(&element_ref, output);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_to_markdown_conversion() {
        let html = "<b>Better than Thanos??</b><br>Marvel somehow never miss when it comes to mainstream villains <a href=\"https://www.tiktok.com/tag/ultron\">#ultron</a> <a href=\"https://www.tiktok.com/tag/villianspeach\">#villianspeach</a><br><br><b>❤️ 537 💬 3 🔁 13</b>";
        let markdown = html_to_markdown(html);
        assert_eq!(
            markdown,
            "**Better than Thanos??**\nMarvel somehow never miss when it comes to mainstream villains #ultron #villianspeach\n\n**❤️ 537 💬 3 🔁 13**"
        );
    }
}
