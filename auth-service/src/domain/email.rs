use validator::ValidateEmail;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(email: &str) -> Result<Self, String> {
        if !ValidateEmail::validate_email(&email) {
            return Err(format!("{} is not a valid email.", email));
        }
        Ok(Email(email.to_owned()))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

//RUST_LOG=info cargo test email::tests -- --nocapture
#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::{DomainSuffix, FreeEmail};
    use fake::Fake;
    //use log::info;

    fn init() {
        env_logger::builder().is_test(true).try_init().ok();
    }

    #[test]
    fn parse_valid_email() {
        init();
        let email = FreeEmail().fake::<String>();
        //info!("testing email: {}", email);
        let result = Email::parse(&email);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Email(email));
    }

    #[test]
    fn parse_invalid_email_no_at() {
        let email_suffix = DomainSuffix().fake::<String>();
        //info!("testing email: {}", email_suffix);
        let result = Email::parse(&email_suffix);
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_email_starts_with_at() {
        let email = format!("@{}", FreeEmail().fake::<String>());
        //info!("testing email: {}", email);
        let result = Email::parse(&email);
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
        Email::parse(&valid_email.0).is_ok()
    }
}
