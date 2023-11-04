use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl AsRef<str> for SubscriberName {
    ///
    /// Return the moved underlying string
    ///
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberName {
    ///
    /// Parse string and check if the string does not contain forbidden characters
    /// - '/', '(', ')', '"', '<', '>', '\\', '{', '}'
    /// - is not empty
    /// - it not greater than 256
    ///
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            return Err(format!("{} is not a valid subscriber name.", s));
        }

        Ok(Self(s))
    }

    ///
    /// Return a moved value of the string
    ///
    pub fn inner(self) -> String {
        self.0
    }

    ///
    /// Return a mutable moved value of the string reference
    ///
    pub fn inner_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_257_grapheme_long_name_is_invalid() {
        let name = "ё".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_invalid() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn empty_string_is_invalid() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn names_containing_an_invalid_character_are_invalid() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }
    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Lorem ipsum".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
