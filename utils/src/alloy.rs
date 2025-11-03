use alloy::{
    consensus::{Signed, TxEip1559, TxEnvelope},
    primitives::{Address, Bytes},
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        Identity, Provider, ProviderBuilder,
    },
    rlp::{self, BytesMut, Encodable},
};

pub type AlloyProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    alloy::providers::RootProvider,
>;

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

pub trait TxExt {
    fn to_raw(self) -> crate::Result<Bytes>;
}

impl TxExt for Signed<TxEip1559> {
    // TODO upstream this to alloy
    fn to_raw(self) -> crate::Result<Bytes> {
        let mut out = BytesMut::new();
        let tx_typed = TxEnvelope::Eip1559(self);
        tx_typed.encode(&mut out);
        Ok(rlp::decode_exact::<Bytes>(out)?)
    }
}
