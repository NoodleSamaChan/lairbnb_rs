use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct LairImage(String);

impl LairImage {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.  
    /// It panics otherwise.
    pub fn parse(s: String) -> Result<LairImage, String> {
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
            Err(format!("{} is not a valid lair image.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for LairImage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::LairImage;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_100000_grapheme_long_image_is_valid() {
        let image = "a̐".repeat(100000);
        assert_ok!(LairImage::parse(image));
    }

    #[test]
    fn an_image_longer_than_100000_graphemes_is_rejected() {
        let image = "a".repeat(100001);
        assert_err!(LairImage::parse(image));
    }

    #[test]
    fn whitespace_only_images_are_rejected() {
        let image = " ".to_string();
        assert_err!(LairImage::parse(image));
    }

    #[test]
    fn empty_string_is_rejected() {
        let image = "".to_string();
        assert_err!(LairImage::parse(image));
    }

    #[test]
    fn a_valid_image_is_parsed_successfully() {
        let image = "Welcome to lair Kefir".to_string();
        assert_ok!(LairImage::parse(image));
    }
}
