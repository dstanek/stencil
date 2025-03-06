use core::fmt;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;

use textwrap::dedent;

#[derive(Debug)]
pub struct ExtractionError;

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Extraction error")
    }
}

impl Error for ExtractionError {}

pub fn extract_blocks(text: &str) -> Result<HashMap<String, String>, ExtractionError> {
    // TODO: this is terribly written :-( fix me
    let start_re =
        Regex::new(r"((?m)^\s*(?<prefix>[^ ]+) begin\-user\-content:(?<id>\S+).*$)").unwrap();

    let mut results = HashMap::new();

    let start_positions = get_start_positions(text, start_re);

    // Iterate through start positions and extract content
    for (id, prefix, start_pos) in start_positions {
        let end_re = Regex::new(&format!(
            r"(?m)^\s*{} end\-user\-content:(\S+).*$",
            regex::escape(prefix),
        ))
        .map_err(|_| ExtractionError)?;

        if let Some(end_match) = end_re.find_at(text, start_pos) {
            let extracted = &text[start_pos..end_match.end()]; // Extract content
            let mut extracted = dedent(extracted).to_string();
            remove_after_last_newline(&mut extracted);
            results.insert(id.to_string(), extracted.to_string());
        }
    }

    Ok(results)
}

fn get_start_positions(text: &str, start_re: Regex) -> Vec<(&str, &str, usize)> {
    let mut start_positions = Vec::new();

    // Collect start marker positions
    for cap in start_re.captures_iter(text) {
        if let Some(m) = cap.name("id") {
            start_positions.push((
                m.as_str(),
                cap.name("prefix").unwrap().as_str(),
                cap.get(1).unwrap().end() + 1,
            ));
            // +1 for the newline
        }
    }
    start_positions
}

fn remove_after_last_newline(s: &mut String) {
    if let Some(last_newline) = s.rfind('\n') {
        s.truncate(last_newline + 1);
    }
}

mod tests {
    #![allow(unused_imports)]
    use super::*;

    #[test]
    fn test_extract_blocks_python() {
        let text = dedent(
            r#"
            # begin-user-content:foo
            This is inside foo block.
            It spans multiple lines.
            # end-user-content:foo

            Some text in between.

            # begin-user-content:bar
            Another block with text.
            # end-user-content:bar
        "#,
        );

        let expected = HashMap::from([
            (
                "foo".to_string(),
                "This is inside foo block.\nIt spans multiple lines.\n".to_string(),
            ),
            ("bar".to_string(), "Another block with text.\n".to_string()),
        ]);

        match extract_blocks(&text) {
            Ok(observed) => assert_eq!(observed, expected),
            Err(e) => panic!("Error: {}", e),
        };
    }

    #[test]
    fn test_extract_blocks_javascript() {
        let text = dedent(
            r#"
            // begin-user-content:foo
            This is inside foo block.
            It spans multiple lines.
            // end-user-content:foo

            Some text in between.

            // begin-user-content:bar
            Another block with text.
            // end-user-content:bar
            "#,
        );

        let expected = HashMap::from([
            (
                "foo".to_string(),
                "This is inside foo block.\nIt spans multiple lines.\n".to_string(),
            ),
            ("bar".to_string(), "Another block with text.\n".to_string()),
        ]);

        match extract_blocks(&text) {
            Ok(observed) => assert_eq!(observed, expected),
            Err(e) => panic!("Error: {}", e),
        };
    }

    #[test]
    fn test_extract_blocks_html() {
        let text = dedent(
            r#"
            <html>
                <script type="text/javascript">
                    // begin-user-content:baz
                    var i = 0;
                    var j = 0;
                    // end-user-content:baz
                </script>
                <body>
                    <!-- begin-user-content:foo -->
                    This is inside foo block.
                    It spans multiple lines.
                    <!-- end-user-content:foo -->

                    Some text in between.

                    <!-- begin-user-content:bar -->
                    Another block with text.
                    <!-- end-user-content:bar -->
                </body>
            </html>
            "#,
        );
        println!("template: {}", text);

        let expected = HashMap::from([
            (
                "foo".to_string(),
                "This is inside foo block.\nIt spans multiple lines.\n".to_string(),
            ),
            ("bar".to_string(), "Another block with text.\n".to_string()),
            ("baz".to_string(), "var i = 0;\nvar j = 0;\n".to_string()),
        ]);

        match extract_blocks(&text) {
            Ok(observed) => assert_eq!(observed, expected),
            Err(e) => panic!("Error: {}", e),
        };
    }

    #[test]
    fn test_remove_after_last_newline() {
        let mut s = String::from("Hello\nWorld\n");
        remove_after_last_newline(&mut s);
        assert_eq!(s, "Hello\nWorld\n");

        let mut s = String::from("Hello\nWorld");
        remove_after_last_newline(&mut s);
        assert_eq!(s, "Hello\n");

        let mut s = String::from("Hello");
        remove_after_last_newline(&mut s);
        assert_eq!(s, "Hello");
    }
}
