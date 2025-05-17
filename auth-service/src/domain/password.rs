use color_eyre::Result;
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, SecretString};
use sqlx::Type;
#[derive(Debug, Clone, Type)]
pub struct Password(SecretString);

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Password {
    pub fn parse(password: SecretString) -> Result<Self> {
        if password.expose_secret().len() < 8 {
            return Err(eyre!("Failed to parse string to a Password type"));
        }
        Ok(Self(password))
    }
}

impl From<SecretString> for Password {
    fn from(s: SecretString) -> Self {
        Self(s)
    }
}

impl AsRef<SecretString> for Password {
    fn as_ref(&self) -> &SecretString {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_empty_password() {
        let pw = SecretString::from("");
        assert!(Password::parse(pw).is_err());
    }
    #[test]
    pub fn test_password_length_less_then_8() {
        let pw = SecretString::from("1");
        assert!(Password::parse(pw).is_err());
    }
    #[test]
    pub fn test_valid_password() {
        let pw = SecretString::from("1".repeat(8));
        assert!(Password::parse(pw).is_ok());
    }
}
