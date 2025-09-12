use alloy::{hex, signers::k256::FieldBytes};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum Secret {
    Mnemonic(String),
    PrivateKey(FieldBytes),
}

impl Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Secret::Mnemonic(mnemonic) => serializer.serialize_str(mnemonic),
            Secret::PrivateKey(private_key) => {
                let hex = hex::encode(private_key);
                serializer.serialize_str(&hex)
            }
        }
    }
}

impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() == 64 {
            let mut bytes = [0u8; 32];
            hex::decode_to_slice(&s, &mut bytes).map_err(serde::de::Error::custom)?;
            Ok(Secret::PrivateKey(*FieldBytes::from_slice(
                bytes.as_slice(),
            )))
        } else {
            Ok(Secret::Mnemonic(s))
        }
    }
}
