use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
};

pub trait StringExt {
    fn parse_as_address(&self) -> crate::Result<Address>;

    fn to_alloy_provider(&self) -> crate::Result<impl Provider>;
}

impl StringExt for str {
    fn parse_as_address(&self) -> crate::Result<Address> {
        self.parse::<Address>()
            .map_err(|_| crate::Error::InvalidAddress(self.to_string()))
    }

    fn to_alloy_provider(&self) -> crate::Result<impl Provider> {
        self.parse()
            .map_err(|e| crate::Error::UrlParsingFailed(self.to_string(), e))
            .map(|rpc_url| ProviderBuilder::new().connect_http(rpc_url))
    }
}

impl StringExt for String {
    fn parse_as_address(&self) -> crate::Result<Address> {
        self.as_str().parse_as_address()
    }

    fn to_alloy_provider(&self) -> crate::Result<impl Provider> {
        self.parse()
            .map_err(|e| crate::Error::UrlParsingFailed(self.to_string(), e))
            .map(|rpc_url| ProviderBuilder::new().connect_http(rpc_url))
    }
}
