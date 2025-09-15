pub type SerdePathError = serde_path_to_error::Error<serde_json::Error>;

pub trait SerdeResponseParse {
    type Error;

    fn serde_parse_custom<T>(
        self,
    ) -> impl std::future::Future<Output = Result<T, Self::Error>> + Send
    where
        T: serde::de::DeserializeOwned;
}

impl SerdeResponseParse for reqwest::Response {
    type Error = crate::Error;

    async fn serde_parse_custom<T>(self) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(serde_response_parse::<T>(&self.text().await?).await?)
    }
}

impl SerdeResponseParse for &str {
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
