use serde_json::Value;

pub type SerdePathError = serde_path_to_error::Error<serde_json::Error>;

pub trait SerdeResponseParse {
    type Error;

    fn serde_parse_custom<T>(self) -> Result<T, Self::Error>
    where
        T: serde::de::DeserializeOwned;
}

impl SerdeResponseParse for Value {
    type Error = crate::Error;

    fn serde_parse_custom<T>(self) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        (&self).serde_parse_custom()
    }
}

impl SerdeResponseParse for &Value {
    type Error = crate::Error;

    fn serde_parse_custom<T>(self) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        if let Ok(s) = serde_json::to_string(&self) {
            // TODO is there a direct way to feed `Value` to serde_path_to_error?
            Ok(serde_path_to_error::deserialize(
                &mut serde_json::Deserializer::from_str(&s),
            )?)
        } else {
            serde_json::from_value(self.clone())
                .map_err(|e| crate::Error::SerdeJsonValueParseFailed(self.clone(), e))
        }
    }
}

pub trait SerdeResponseParseAsync {
    type Error;

    fn serde_parse_custom<T>(
        self,
    ) -> impl std::future::Future<Output = Result<T, Self::Error>> + Send
    where
        T: serde::de::DeserializeOwned;
}

impl SerdeResponseParseAsync for reqwest::Response {
    type Error = crate::Error;

    async fn serde_parse_custom<T>(self) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(serde_response_parse::<T>(&self.text().await?).await?)
    }
}

impl SerdeResponseParseAsync for &str {
    type Error = SerdePathError;

    async fn serde_parse_custom<T>(self) -> Result<T, SerdePathError>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_response_parse::<T>(self).await
    }
}

async fn serde_response_parse<T>(s: &str) -> Result<T, SerdePathError>
where
    T: serde::de::DeserializeOwned,
{
    serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(s))
}
