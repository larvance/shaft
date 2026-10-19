use std::path::Path;
use pulldown_cmark::{Event, Options, Parser, TagEnd};
use shaft_core::normalize_format;

pub fn markdown_to_html(markdown_input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown_input, options);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
}

pub fn markdown_to_plaintext(markdown_input: &str) -> String {
    let parser = Parser::new(markdown_input);
    let mut out = String::new();

    for event in parser {
        match event {
            Event::Text(t) => out.push_str(&t),
            Event::Code(c) => out.push_str(&c),
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Heading(_)) => {
                out.push_str("\n\n");
            }
            Event::End(TagEnd::Item) => {
                out.push('\n');
            }
            _ => {}
        }
    }

    out.trim().to_string()
}

pub fn transcode_document<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    output_path: Q,
    target_format: &str,
) -> Result<(), String> {
    let src = source_path.as_ref();
    let dst = output_path.as_ref();

    if !src.exists() {
        return Err(format!("Document file does not exist: {}", src.display()));
    }

    let input_text = std::fs::read_to_string(src)
        .map_err(|e| format!("Failed to read source document: {}", e))?;

    let norm_target = normalize_format(target_format);
    let output_text = match norm_target.as_str() {
        "html" | "htm" => {
            let body = markdown_to_html(&input_text);
            format!(
                "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>Document</title>\n</head>\n<body>\n{}\n</body>\n</html>\n",
                body
            )
        }
        "txt" => markdown_to_plaintext(&input_text),
        "md" => input_text,
        _ => {
            // Default to plain text
            markdown_to_plaintext(&input_text)
        }
    };

    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    std::fs::write(dst, output_text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html() {
        let md = "# Title\n\nThis is **bold** text and `code`.\n";
        let html = markdown_to_html(md);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_markdown_to_plaintext() {
        let md = "# Title\n\nThis is **bold** text and [link](https://aerovex.net).\n";
        let plain = markdown_to_plaintext(md);
        assert!(plain.contains("Title"));
        assert!(plain.contains("This is bold text and link."));
        assert!(!plain.contains("#"));
        assert!(!plain.contains("**"));
    }
}
