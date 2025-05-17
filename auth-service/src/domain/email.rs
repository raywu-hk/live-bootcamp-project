use color_eyre::Result;
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::hash::Hash;
use validator::ValidateEmail;
#[derive(Deserialize, Debug, Clone)]
pub struct Email(SecretString);

impl Email {
    pub fn parse(email: SecretString) -> Result<Self> {
        if !ValidateEmail::validate_email(&email.expose_secret()) {
            return Err(eyre!("{} is not a valid email.", email.expose_secret()));
        }
        Ok(Email(email))
    }
}
impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}
impl Eq for Email {}
impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl AsRef<SecretString> for Email {
    fn as_ref(&self) -> &SecretString {
        &self.0
    }
}

//RUST_LOG=info cargo test email::tests -- --nocapture
#[cfg(test)]
mod tests {
    use crate::Email;
    use fake::Fake;
    use fake::faker::internet::en::{DomainSuffix, FreeEmail};
    use secrecy::SecretString;
    //use log::info;

    fn init() {
        env_logger::builder().is_test(true).try_init().ok();
    }

    #[test]
    fn parse_valid_email() {
        init();
        let email = FreeEmail().fake::<String>();
        let sec_email = SecretString::from(email);
        //info!("testing email: {}", email);
        let result = Email::parse(sec_email.clone()).unwrap();
        assert_eq!(result, Email::parse(sec_email.clone()).unwrap());
    }

    #[test]
    fn parse_invalid_email_no_at() {
        let email_suffix = DomainSuffix().fake::<String>();
        //info!("testing email: {}", email_suffix);
        let result = Email::parse(SecretString::from(email_suffix));
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_email_starts_with_at() {
        let email = format!("@{}", FreeEmail().fake::<String>());
        //info!("testing email: {}", email);
        let result = Email::parse(SecretString::from(email));
        assert!(result.is_err());
    }
    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = FreeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(SecretString::from(valid_email.0)).is_ok()
    }
}
