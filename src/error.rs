#[derive(Debug)]
pub enum Error {
    AppleSecurityFrameworkError(security_framework::base::Error),
    InquireError(inquire::InquireError),
    AlloyEcdsaError(alloy::signers::k256::ecdsa::Error),
}

impl From<security_framework::base::Error> for Error {
    fn from(e: security_framework::base::Error) -> Self {
        Error::AppleSecurityFrameworkError(e)
    }
}

impl From<inquire::InquireError> for Error {
    fn from(e: inquire::InquireError) -> Self {
        Error::InquireError(e)
    }
}

impl From<alloy::signers::k256::ecdsa::Error> for Error {
    fn from(e: alloy::signers::k256::ecdsa::Error) -> Self {
        Error::AlloyEcdsaError(e)
    }
}
