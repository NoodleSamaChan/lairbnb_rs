use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct LairTitle(String);

impl LairTitle {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.  
    /// It panics otherwise.
    pub fn parse(s: String) -> Result<LairTitle, String> {
        // `.trim()` returns a view over the input `s` without trailing
        // whitespace-like characters.
        // `.is_empty` checks if the view contains any character.
        let is_empty_or_whitespace = s.trim().is_empty();

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        let is_too_long = s.graphemes(true).count() > 1000;

        if is_empty_or_whitespace || is_too_long {
            Err(format!("{} is not a valid lair title.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for LairTitle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::LairTitle;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_title_is_valid() {
        let title = "a̐".repeat(256);
        assert_ok!(LairTitle::parse(title));
    }

    #[test]
    fn a_title_longer_than_256_graphemes_is_rejected() {
        let title = "a".repeat(257);
        assert_err!(LairTitle::parse(title));
    }

    #[test]
    fn whitespace_only_titles_are_rejected() {
        let title = " ".to_string();
        assert_err!(LairTitle::parse(title));
    }

    #[test]
    fn empty_string_is_rejected() {
        let title = "".to_string();
        assert_err!(LairTitle::parse(title));
    }

    #[test]
    fn a_valid_title_is_parsed_successfully() {
        let title = "Welcome to lair Kefir".to_string();
        assert_ok!(LairTitle::parse(title));
    }
}