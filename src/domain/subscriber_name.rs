use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn inner(self) -> String {
        self.0
    }

    pub fn inner_mut(&mut self) -> &mut str {
        &mut self.0
    }

    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));
        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::subscriber_name::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        // Arrange
        let name = "ё".repeat(256);

        // Act
        let result = SubscriberName::parse(name);

        // Assert
        assert_ok!(result);
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        // Arrange
        let name = "ё".repeat(257);

        // Act
        let result = SubscriberName::parse(name);

        // Assert
        assert_err!(result);
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        // Arrange
        let name = " ".to_string();

        // Act
        let result = SubscriberName::parse(name);

        // Assert
        assert_err!(result);
    }

    #[test]
    fn empty_string_is_rejected() {
        // Arrange
        let name = "".to_string();

        // Act
        let result = SubscriberName::parse(name);

        // Assert
        assert_err!(result);
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        // Arrange
        let invalid_characters = &['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

        for name in invalid_characters {
            let name = name.to_string();

            // Act
            let result = SubscriberName::parse(name);

            // Assert
            assert_err!(result);
        }
    }
    #[test]
    fn a_valid_name_is_parsed_successfully() {
        // Arrange
        let name = "Ursula Le Guin".to_string();

        // Act
        let result = SubscriberName::parse(name);

        // Assert
        assert_ok!(result);
    }
}
