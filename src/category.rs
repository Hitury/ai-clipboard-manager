use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Category {
    Text,
    Link,
    Code,
    Path,
    Email,
    Color,
    Number,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Link => "Links",
            Self::Code => "Code",
            Self::Path => "Paths",
            Self::Email => "Emails",
            Self::Color => "Colors",
            Self::Number => "Numbers",
        }
    }

    /// Classifies clipboard text with cheap heuristics, most specific first.
    pub fn detect(text: &str) -> Self {
        let t = text.trim();
        let word = !t.contains(char::is_whitespace);
        if word && ["http://", "https://", "ftp://", "www."].iter().any(|p| t.starts_with(p)) {
            Self::Link
        } else if word && is_email(t) {
            Self::Email
        } else if is_color(t) {
            Self::Color
        } else if t.bytes().any(|b| b.is_ascii_digit()) && t.replace([',', '_'], "").parse::<f64>().is_ok() {
            Self::Number
        } else if !t.contains('\n') && is_path(t) {
            Self::Path
        } else if is_code(t) {
            Self::Code
        } else {
            Self::Text
        }
    }
}

fn is_email(t: &str) -> bool {
    t.split_once('@').is_some_and(|(user, domain)| {
        !user.is_empty() && !domain.contains('@') && domain.trim_matches('.').contains('.')
    })
}

fn is_color(t: &str) -> bool {
    let hex = t
        .strip_prefix('#')
        .is_some_and(|h| matches!(h.len(), 3 | 4 | 6 | 8) && h.chars().all(|c| c.is_ascii_hexdigit()));
    let func = ["rgb(", "rgba(", "hsl(", "hsla("].iter().any(|p| t.starts_with(p)) && t.ends_with(')');
    hex || func
}

fn is_path(t: &str) -> bool {
    let b = t.as_bytes();
    let drive = b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && matches!(b[2], b'\\' | b'/');
    let unix = t.len() > 1 && t.starts_with('/') && !t.starts_with("//");
    drive || unix || ["~/", "./", "../", "\\\\"].iter().any(|p| t.starts_with(p))
}

fn is_code(t: &str) -> bool {
    const HINTS: [&str; 14] = [
        "fn ", "let ", "const ", "var ", "def ", "function", "import ", "#include", "return ", "=>", "();",
        "{\n", "</", "::",
    ];
    let wrapped = matches!(
        (t.chars().next(), t.chars().last()),
        (Some('{'), Some('}')) | (Some('['), Some(']')) | (Some('<'), Some('>'))
    );
    let symbols = t.chars().filter(|c| "{}()[];=<>".contains(*c)).count();
    wrapped || (symbols * 25 > t.len() && HINTS.iter().any(|h| t.contains(h)))
}

#[cfg(test)]
mod tests {
    use super::Category::{self, *};

    #[test]
    fn detects_categories() {
        let cases = [
            ("https://example.com/a?b=1", Link),
            ("dev@example.org", Email),
            ("#38bdf8", Color),
            ("rgb(15, 23, 42)", Color),
            ("1,234.50", Number),
            ("C:\\Users\\me\\notes.txt", Path),
            ("~/projects/app", Path),
            ("let x = foo();", Code),
            ("{\"a\": 1}", Code),
            ("Just a regular sentence (with parens).", Text),
            ("inf", Text),
        ];
        for (input, expected) in cases {
            assert_eq!(Category::detect(input), expected, "{input}");
        }
    }
}
