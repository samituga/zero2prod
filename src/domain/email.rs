use std::fmt::{Display, Formatter};
use validator::ValidateEmail;

#[derive(Debug, Clone, PartialEq)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Email, String> {
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email.", s))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use proptest::prelude::{any, Strategy};
    use proptest::proptest;

    use super::Email;

    #[test]
    fn empty_string_is_rejected() {
        // Arrange
        let email = "".to_string();

        // Act
        let result = Email::parse(email);

        // Assert
        assert_err!(result);
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        // Arrange
        let email = "ursuladomain.com".to_string();

        // Act
        let result = Email::parse(email);

        // Assert
        assert_err!(result);
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        // Arrange
        let email = "@domain.com".to_string();

        // Act
        let result = Email::parse(email);

        // Arrange
        assert_err!(result);
    }

    fn valid_email_strategy() -> impl Strategy<Value = String> {
        any::<String>().prop_map(|_| SafeEmail().fake())
    }

    proptest! {
        #[test]
        fn valid_emails_are_parsed_successfully(email in valid_email_strategy()) {
            // Act
            let result = Email::parse(email.clone());

            // Assert
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_ref(), email);
        }
    }
}
