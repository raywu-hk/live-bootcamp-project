use color_eyre::Result;
use color_eyre::eyre::eyre;
use sqlx::Type;
#[derive(Debug, PartialEq, Clone, Eq, Type)]
#[sqlx(transparent)]
pub struct Password(String);

impl Password {
    pub fn parse(password: &str) -> Result<Self> {
        if password.len() < 8 {
            return Err(eyre!("Failed to parse string to a Password type"));
        }
        Ok(Self(password.to_string()))
    }
}

impl From<String> for Password {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_empty_password() {
        assert!(Password::parse("").is_err());
    }
    #[test]
    pub fn test_password_length_less_then_8() {
        assert!(Password::parse("1").is_err());
    }
    #[test]
    pub fn test_valid_password() {
        assert!(Password::parse("1".repeat(8).as_str()).is_ok());
    }
}
