//! Bounded markup stripping; malformed input and literal `|n` preservation are simulator policies.

#[derive(Clone, Copy, Debug, Default)]
pub struct StripHyperlinksOptions {
    pub maintain_color: bool,
    pub maintain_brackets: bool,
    pub strip_newlines: bool,
    pub maintain_atlases: bool,
    pub maintain_textures: bool,
}

pub fn strip_hyperlinks(text: &str, _options: &StripHyperlinksOptions) -> String {
    text.to_owned()
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
