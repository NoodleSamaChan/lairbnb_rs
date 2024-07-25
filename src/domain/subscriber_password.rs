use unicode_segmentation::UnicodeSegmentation;
#[derive(Debug)]
pub struct SubscriberPassword(String);

impl SubscriberPassword {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.  
    /// It panics otherwise.
    pub fn parse(s: String) -> Result<SubscriberPassword, String> {
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
        let is_too_long = s.graphemes(true).count() > 256;

        let is_too_short = s.graphemes(true).count() < 6;

        // Iterate over all characters in the input `s` to check if any of them matches
        // one of the characters in the forbidden array.
        let special_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_special_characters = s.chars().any(|g| special_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || is_too_short || !contains_special_characters {
            Err(format!("{} is not a valid subscriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberPassword;
    use claims::assert_err;

    #[test]
    fn empty_string_is_rejected() {
        let password = "".to_string();
        assert_err!(SubscriberPassword::parse(password));
    }

    #[test]
    fn a_password_shorter_than_6_graphemes_is_rejected() {
        let name = "!".repeat(6);
        assert_err!(SubscriberPassword::parse(name));
    }
}