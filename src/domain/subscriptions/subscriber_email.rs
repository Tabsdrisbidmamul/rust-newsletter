use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberEmail {
    ///
    /// Parse sting to check if its a valid email
    ///
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if !validate_email(&s) {
            return Err(format!("{} is not a valid a subscriber email.", s));
        }

        Ok(Self(s))
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::Arbitrary;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn empty_string_is_invalid() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_invalid() {
        let email = "testtest.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_invalid() {
        let email = "@test.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
