pub trait SerdeResponseParse {
    fn serde_parse_custom<T>(self) -> impl std::future::Future<Output = crate::Result<T>> + Send
    where
        T: serde::de::DeserializeOwned;
}

impl SerdeResponseParse for reqwest::Response {
    async fn serde_parse_custom<T>(self) -> crate::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_response_parse::<T>(self).await
    }
}

async fn serde_response_parse<T>(response: reqwest::Response) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_path_to_error::deserialize(
        &mut serde_json::Deserializer::from_str(&response.text().await?),
    )?)
}
