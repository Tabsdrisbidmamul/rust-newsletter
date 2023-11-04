use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if !validate_email(&s) {
            return Err(format!("{} is not a valid a subscriber email.", s));
        }

        Ok(Self(s))
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;

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
