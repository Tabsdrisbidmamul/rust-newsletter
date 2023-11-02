use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    ///
    /// Parse string and check if the string does not contain forbidden characters
    /// - '/', '(', ')', '"', '<', '>', '\\', '{', '}'
    /// - is not empty
    /// - it not greater than 256
    ///
    pub fn parse(s: String) -> SubscriberName {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            panic!("{} is not a valid subscriber name.", s)
        } else {
            Self(s)
        }
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

    ///
    /// Return a moved reference of the string
    ///
    pub fn inner_ref(&self) -> &str {
        &self.0
    }
}
