//! Note bodies as data: the Markdown a slip-box uses, plus `[[wikilinks]]` and
//! `#tags`, as the desktop page reads it (`desktop/dist/js/md.js`). The screens
//! turn these into text nodes; nothing a note holds is ever parsed as HTML.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(Vec<Inline>),
    Paragraph(Vec<Inline>),
    Quote(Vec<Inline>),
    List(Vec<Vec<Inline>>),
    Code(String),
    Rule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Code(String),
    Strong(String),
    Em(String),
    /// `[[target]]` or `[[target|alias]]`: the target (a zid or a title) and the words shown.
    Wiki {
        target: String,
        text: String,
    },
    /// `#tag`, lowercased.
    Tag(String),
    /// `[label](https://…)` or a bare URL; only http(s) and mailto.
    Link {
        href: String,
        text: String,
    },
}

/// Split a body into blocks.
#[must_use]
pub fn parse(body: &str) -> Vec<Block> {
    let lines: Vec<&str> = body.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            i += 1;
        } else if line.starts_with("```") {
            let mut code = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                code.push(lines[i]);
                i += 1;
            }
            i += 1;
            blocks.push(Block::Code(code.join("\n")));
        } else if let Some(text) = heading(line) {
            blocks.push(Block::Heading(inline(text)));
            i += 1;
        } else if is_rule(line) {
            blocks.push(Block::Rule);
            i += 1;
        } else if line.starts_with('>') {
            let mut quote = Vec::new();
            while i < lines.len() && lines[i].starts_with('>') {
                quote.push(lines[i].trim_start_matches('>').trim_start());
                i += 1;
            }
            blocks.push(Block::Quote(inline(&quote.join(" "))));
        } else if item(line).is_some() {
            let mut items = Vec::new();
            while let Some(text) = lines.get(i).and_then(|l| item(l)) {
                items.push(inline(text));
                i += 1;
            }
            blocks.push(Block::List(items));
        } else {
            let mut para = Vec::new();
            while i < lines.len() {
                let l = lines[i];
                if l.trim().is_empty()
                    || l.starts_with("```")
                    || l.starts_with('>')
                    || heading(l).is_some()
                    || item(l).is_some()
                {
                    break;
                }
                para.push(l.trim());
                i += 1;
            }
            blocks.push(Block::Paragraph(inline(&para.join(" "))));
        }
    }
    blocks
}

fn heading(line: &str) -> Option<&str> {
    let hashes = line.len() - line.trim_start_matches('#').len();
    ((1..=6).contains(&hashes) && line[hashes..].starts_with(' ')).then(|| line[hashes..].trim())
}

fn is_rule(line: &str) -> bool {
    let t = line.trim_end();
    t.len() >= 3 && (t.chars().all(|c| c == '-') || t.chars().all(|c| c == '*'))
}

fn item(line: &str) -> Option<&str> {
    let t = line.trim_start();
    t.strip_prefix("- ")
        .or_else(|| t.strip_prefix("* "))
        .or_else(|| {
            let digits = t.len() - t.trim_start_matches(|c: char| c.is_ascii_digit()).len();
            (digits > 0)
                .then(|| t[digits..].strip_prefix(". "))
                .flatten()
        })
}

/// Split a line into text and the marks the slip-box knows.
#[must_use]
pub fn inline(text: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    let mut prev: Option<char> = None;
    while let Some(c) = rest.chars().next() {
        if let Some((mark, used)) = mark_at(rest, prev) {
            if !plain.is_empty() {
                out.push(Inline::Text(std::mem::take(&mut plain)));
            }
            out.push(mark);
            prev = rest[..used].chars().last();
            rest = &rest[used..];
        } else {
            plain.push(c);
            prev = Some(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    if !plain.is_empty() {
        out.push(Inline::Text(plain));
    }
    out
}

/// The mark that starts `s`, and how many bytes it takes.
fn mark_at(s: &str, prev: Option<char>) -> Option<(Inline, usize)> {
    if let Some(body) = s.strip_prefix("[[") {
        let end = body
            .find("]]")
            .filter(|&e| e > 0 && !body[..e].contains('\n'))?;
        let raw = &body[..end];
        let (target, alias) = raw.split_once('|').unwrap_or((raw, raw));
        let target = target
            .split('#')
            .next()
            .unwrap_or(target)
            .trim()
            .to_string();
        return Some((
            Inline::Wiki {
                target,
                text: alias.trim().to_string(),
            },
            end + 4,
        ));
    }
    if let Some(body) = s.strip_prefix('`') {
        let end = body.find('`').filter(|&e| e > 0)?;
        return Some((Inline::Code(body[..end].to_string()), end + 2));
    }
    if let Some(body) = s.strip_prefix("**") {
        let end = body.find("**").filter(|&e| e > 0)?;
        return Some((Inline::Strong(body[..end].to_string()), end + 4));
    }
    for d in ['*', '_'] {
        if let Some(body) = s.strip_prefix(d) {
            if body.starts_with(char::is_whitespace) || body.starts_with(d) {
                continue;
            }
            if let Some(end) = body.find(d).filter(|&e| e > 0) {
                return Some((Inline::Em(body[..end].to_string()), end + 2));
            }
        }
    }
    if s.starts_with('[') {
        let close = s.find("](")?;
        let end = s[close..].find(')')? + close;
        let href = &s[close + 2..end];
        if safe(href) && !href.contains(char::is_whitespace) {
            return Some((
                Inline::Link {
                    href: href.to_string(),
                    text: s[1..close].to_string(),
                },
                end + 1,
            ));
        }
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        let end = s
            .find(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == ')')
            .unwrap_or(s.len());
        return Some((
            Inline::Link {
                href: s[..end].to_string(),
                text: s[..end].to_string(),
            },
            end,
        ));
    }
    if let Some(body) = s.strip_prefix('#') {
        if prev.is_some_and(|p| !p.is_whitespace() && p != '(') {
            return None;
        }
        let end = body
            .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '/' || c == '-'))
            .unwrap_or(body.len());
        let tag = &body[..end];
        if tag.chars().any(char::is_alphabetic) {
            return Some((Inline::Tag(tag.to_lowercase()), end + 1));
        }
    }
    None
}

fn safe(href: &str) -> bool {
    let h = href.to_ascii_lowercase();
    h.starts_with("http://") || h.starts_with("https://") || h.starts_with("mailto:")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Inline {
        Inline::Text(s.into())
    }

    #[test]
    fn wikilinks_and_tags() {
        assert_eq!(
            inline("Continued in [[1a|One idea per note]]. #pkm #2024 (#Writing)"),
            vec![
                text("Continued in "),
                Inline::Wiki {
                    target: "1a".into(),
                    text: "One idea per note".into()
                },
                text(". "),
                Inline::Tag("pkm".into()),
                text(" #2024 ("),
                Inline::Tag("writing".into()),
                text(")"),
            ]
        );
        assert_eq!(
            inline("see [[Links beat folders#why]]"),
            vec![
                text("see "),
                Inline::Wiki {
                    target: "Links beat folders".into(),
                    text: "Links beat folders#why".into()
                }
            ]
        );
    }

    #[test]
    fn no_tag_inside_a_word_and_no_unsafe_link() {
        assert_eq!(inline("C#sharp"), vec![text("C#sharp")]);
        assert_eq!(
            inline("[x](javascript:alert)"),
            vec![text("[x](javascript:alert)")]
        );
    }

    #[test]
    fn marks() {
        assert_eq!(
            inline("**b** *e* `c` https://a.b/c"),
            vec![
                Inline::Strong("b".into()),
                text(" "),
                Inline::Em("e".into()),
                text(" "),
                Inline::Code("c".into()),
                text(" "),
                Inline::Link {
                    href: "https://a.b/c".into(),
                    text: "https://a.b/c".into()
                },
            ]
        );
    }

    #[test]
    fn blocks() {
        let body = "# Title\n\nOne\ntwo\n\n> a quote\n> goes on\n\n- x\n- y\n\n```\nfn main() {}\n```\n---";
        assert_eq!(
            parse(body),
            vec![
                Block::Heading(vec![text("Title")]),
                Block::Paragraph(vec![text("One two")]),
                Block::Quote(vec![text("a quote goes on")]),
                Block::List(vec![vec![text("x")], vec![text("y")]]),
                Block::Code("fn main() {}".into()),
                Block::Rule,
            ]
        );
    }
}
