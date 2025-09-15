use alloy::primitives::Address;

pub trait StringExt {
    fn parse_as_address(&self) -> crate::Result<Address>;
}

impl StringExt for str {
    fn parse_as_address(&self) -> crate::Result<Address> {
        self.parse::<Address>()
            .map_err(|_| crate::Error::InvalidAddress(self.to_string()))
    }
}

impl StringExt for String {
    fn parse_as_address(&self) -> crate::Result<Address> {
        self.as_str().parse_as_address()
    }
}
