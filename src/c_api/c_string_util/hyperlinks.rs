//! Bounded markup stripping; malformed input and literal `|n` preservation are simulator policies.

#[derive(Clone, Copy, Debug, Default)]
pub struct StripHyperlinksOptions {
    pub maintain_color: bool,
    pub maintain_brackets: bool,
    pub strip_newlines: bool,
    pub maintain_atlases: bool,
    pub maintain_textures: bool,
}

pub fn strip_hyperlinks(text: &str, options: &StripHyperlinksOptions) -> String {
    let mut output = String::with_capacity(text.len());
    let mut closers = std::collections::HashSet::new();
    let mut offset = 0;
    while offset < text.len() {
        let remaining = &text[offset..];
        if remaining.starts_with('|') {
            offset += consume_markup(text, offset, options, &mut closers, &mut output);
            continue;
        }
        // offset always lies on a UTF-8 boundary and remaining is nonempty.
        let character = remaining.chars().next().unwrap_or_default();
        if options.maintain_brackets || !matches!(character, '[' | ']') {
            output.push(character);
        }
        offset += character.len_utf8();
    }
    output
}

/// Find a real marker, skipping escaped pipe pairs.
fn find_marker(text: &str, start: usize, marker: u8) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut offset = start;
    while offset + 1 < bytes.len() {
        if bytes[offset] != b'|' {
            offset += 1;
        } else if bytes[offset + 1] == b'|' {
            offset += 2;
        } else if bytes[offset + 1] == marker {
            return Some(offset);
        } else {
            offset += 2;
        }
    }
    None
}

fn consume_markup(
    text: &str,
    offset: usize,
    options: &StripHyperlinksOptions,
    closers: &mut std::collections::HashSet<usize>,
    output: &mut String,
) -> usize {
    let remaining = &text[offset..];
    let code = remaining.as_bytes().get(1).copied();
    if closers.remove(&offset) {
        if code == Some(b'r') && options.maintain_color {
            output.push_str("|r");
        }
        return 2;
    }
    match code {
        Some(b'|') => {
            output.push_str("||");
            2
        }
        Some(b'n') => {
            if !options.strip_newlines {
                output.push_str("|n");
            }
            2
        }
        Some(b'A' | b'T') => consume_image(text, offset, options, output),
        Some(b'H') => {
            let bounds = find_marker(text, offset + 2, b'h')
                .and_then(|start| find_marker(text, start + 2, b'h').map(|end| (start, end)));
            if let Some((start, end)) = bounds {
                closers.insert(end);
                start + 2 - offset
            } else {
                preserve_remainder(remaining, output)
            }
        }
        Some(b'c') => consume_color(text, offset, options, closers, output),
        _ => {
            output.push('|');
            1
        }
    }
}

fn consume_image(
    text: &str,
    offset: usize,
    options: &StripHyperlinksOptions,
    output: &mut String,
) -> usize {
    let atlas = text.as_bytes()[offset + 1] == b'A';
    let marker = if atlas { b'a' } else { b't' };
    let Some(end) = find_marker(text, offset + 2, marker) else {
        return preserve_remainder(&text[offset..], output);
    };
    if (atlas && options.maintain_atlases) || (!atlas && options.maintain_textures) {
        output.push_str(&text[offset..end + 2]);
    }
    end + 2 - offset
}

fn consume_color(
    text: &str,
    offset: usize,
    options: &StripHyperlinksOptions,
    closers: &mut std::collections::HashSet<usize>,
    output: &mut String,
) -> usize {
    let digits = text.as_bytes().get(offset + 2..offset + 10);
    let valid = digits.is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit));
    let end = valid
        .then(|| find_marker(text, offset + 10, b'r'))
        .flatten();
    let Some(end) = end else {
        return preserve_remainder(&text[offset..], output);
    };
    closers.insert(end);
    if options.maintain_color {
        output.push_str(&text[offset..offset + 10]);
    }
    10
}

// An unterminated recognized construct makes its remaining span opaque.
fn preserve_remainder(text: &str, output: &mut String) -> usize {
    output.push_str(text);
    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_wrappers_and_processes_unicode_label() {
        assert_eq!(
            strip_hyperlinks("é |Hitem:12|h[|cFFFFFFFF剣|r]|h!", &Default::default()),
            "é 剣!"
        );
    }

    #[test]
    fn honors_each_preservation_flag() {
        let input = "|cFFFFFFFF[A]|r|Ax:16|a|Ty:16|t";
        for (options, expected) in [
            (StripHyperlinksOptions::default(), "A"),
            (
                StripHyperlinksOptions {
                    maintain_color: true,
                    ..Default::default()
                },
                "|cFFFFFFFFA|r",
            ),
            (
                StripHyperlinksOptions {
                    maintain_brackets: true,
                    ..Default::default()
                },
                "[A]",
            ),
            (
                StripHyperlinksOptions {
                    maintain_atlases: true,
                    ..Default::default()
                },
                "A|Ax:16|a",
            ),
            (
                StripHyperlinksOptions {
                    maintain_textures: true,
                    ..Default::default()
                },
                "A|Ty:16|t",
            ),
        ] {
            assert_eq!(strip_hyperlinks(input, &options), expected);
        }
    }

    #[test]
    fn preserves_native_consumer_categories_inside_link() {
        let options = StripHyperlinksOptions {
            maintain_brackets: true,
            maintain_atlases: true,
            maintain_textures: true,
            ..Default::default()
        };
        assert_eq!(
            strip_hyperlinks("|Hspell:1|h[|cFFFFFFFF火|r|Ax|a|Ty|t]|h", &options),
            "[火|Ax|a|Ty|t]"
        );
    }

    #[test]
    fn handles_newlines_without_changing_real_newlines() {
        assert_eq!(strip_hyperlinks("a|nb\nc", &Default::default()), "a|nb\nc");
        assert_eq!(
            strip_hyperlinks(
                "a|nb\nc",
                &StripHyperlinksOptions {
                    strip_newlines: true,
                    ..Default::default()
                }
            ),
            "ab\nc"
        );
    }

    #[test]
    fn escaped_pipes_do_not_start_markup_or_close_tokens() {
        assert_eq!(
            strip_hyperlinks("||cFFFFFFFF[x]||n", &Default::default()),
            "||cFFFFFFFFx||n"
        );
        assert_eq!(
            strip_hyperlinks("a|Tx||tstill|tb", &Default::default()),
            "ab"
        );
        assert_eq!(
            strip_hyperlinks("|Hitem:1|hA||hB|h", &Default::default()),
            "A||hB"
        );
    }

    #[test]
    fn preserves_malformed_sequences_literally() {
        for input in [
            "|",
            "|Qx",
            "|Tbad[x]",
            "|Abad",
            "|Hitem:1|h[label]",
            "|cBAD[x]",
            "|cFFFFFFFF[x]",
            "|h",
            "|r",
        ] {
            assert_eq!(strip_hyperlinks(input, &Default::default()), input);
        }
    }
}
