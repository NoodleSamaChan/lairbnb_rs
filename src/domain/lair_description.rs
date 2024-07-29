use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct LairDescription(String);

impl LairDescription {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.  
    /// It panics otherwise.
    pub fn parse(s: String) -> Result<LairDescription, String> {
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
        let is_too_long = s.graphemes(true).count() > 100000;

        if is_empty_or_whitespace || is_too_long {
            Err(format!("{} is not a valid lair description.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for LairDescription {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::LairDescription;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_100000_grapheme_long_image_is_valid() {
        let description = "a̐".repeat(100000);
        assert_ok!(LairDescription::parse(description));
    }

    #[test]
    fn an_description_longer_than_100000_graphemes_is_rejected() {
        let description = "a".repeat(257);
        assert_err!(LairDescription::parse(description));
    }

    #[test]
    fn whitespace_only_descriptions_are_rejected() {
        let description = " ".to_string();
        assert_err!(LairDescription::parse(description));
    }

    #[test]
    fn empty_string_is_rejected() {
        let description = "".to_string();
        assert_err!(LairDescription::parse(description));
    }

    #[test]
    fn a_valid_description_is_parsed_successfully() {
        let description = "Welcome to lair Kefir".to_string();
        assert_ok!(LairDescription::parse(description));
    }
}