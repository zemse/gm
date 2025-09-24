use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use url::Url;

use crate::serde::{SerdePathError, SerdeResponseParseAsync};

#[derive(Debug)]
pub enum ReqwestStage {
    Send,
    Status,
    DecodeText,
    Deserialise,
}

fn parse_url<U: ToString>(url: U) -> crate::Result<Url> {
    url.to_string()
        .parse::<Url>()
        .map_err(|_| crate::Error::InvalidUrl(url.to_string()))
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ReqwestErrorContext {
    pub url: Url,
    query: String,
    json_body: String,
}

#[derive(Debug)]
pub enum ReqwestInnerError {
    Reqwest(reqwest::Error),
    SerdePath(SerdePathError),
}

impl ReqwestInnerError {
    pub fn is_connect(&self) -> bool {
        match self {
            Self::Reqwest(err) => err.is_connect(),
            Self::SerdePath(_) => false,
        }
    }
}

pub struct Reqwest {
    builder: RequestBuilder,
    error_context: Option<ReqwestErrorContext>,
}

impl Reqwest {
    pub fn get<U: ToString>(url: U) -> crate::Result<Self> {
        let url = parse_url(url)?;
        let client = reqwest::Client::new();
        let builder = client.get(url.clone());
        Ok(Self {
            builder,
            error_context: Some(ReqwestErrorContext {
                url,
                query: String::new(),
                json_body: String::new(),
            }),
        })
    }

    pub fn post<U: ToString>(url: U) -> crate::Result<Self> {
        let url = parse_url(url)?;
        let client = reqwest::Client::new();
        let builder = client.post(url.clone());
        Ok(Self {
            builder,
            error_context: Some(ReqwestErrorContext {
                url,
                query: String::new(),
                json_body: String::new(),
            }),
        })
    }

    pub fn query<T: serde::Serialize + Debug>(mut self, query: &T) -> Self {
        self.builder = self.builder.query(query);
        self
    }

    pub fn json_body<T: serde::Serialize + Debug>(mut self, json_body: &T) -> Self {
        self.builder = self.builder.json(json_body);
        self
    }

    pub async fn receive_text(self) -> crate::Result<String> {
        let (text, _) = self.receive_text_internal().await?;
        Ok(text)
    }

    async fn receive_text_internal(mut self) -> crate::Result<(String, Box<ReqwestErrorContext>)> {
        let error_context = Box::new(
            self.error_context
                .take()
                .ok_or(crate::Error::ReqwestErrorContextMissing)?,
        );

        let send_result = self.builder.send().await;
        let Ok(response) = send_result else {
            let err = send_result.unwrap_err();
            if err.is_connect() {
                return Err(crate::Error::Internet(error_context.url));
            } else {
                return Err(crate::Error::ReqwestFailed {
                    stage: ReqwestStage::Send,
                    context: error_context,
                    inner: ReqwestInnerError::Reqwest(err),
                });
            }
        };

        let status_result = response.error_for_status();
        let Ok(response) = status_result else {
            let err = status_result.unwrap_err();
            return Err(crate::Error::ReqwestFailed {
                stage: ReqwestStage::Status,
                context: error_context,
                inner: ReqwestInnerError::Reqwest(err),
            });
        };

        let text_result = response.text().await;
        let Ok(text) = text_result else {
            let err = text_result.unwrap_err();
            return Err(crate::Error::ReqwestFailed {
                stage: ReqwestStage::DecodeText,
                context: error_context,
                inner: ReqwestInnerError::Reqwest(err),
            });
        };

        Ok((text, error_context))
    }

    pub async fn receive_json<J: Debug + DeserializeOwned>(self) -> crate::Result<J> {
        let (text, error_context) = self.receive_text_internal().await?;

        let parse_result = text.serde_parse_custom().await;
        let Ok(json) = parse_result else {
            let err = parse_result.unwrap_err();
            return Err(crate::Error::ReqwestFailed {
                stage: ReqwestStage::Deserialise,
                context: error_context,
                inner: ReqwestInnerError::SerdePath(err),
            });
        };
        Ok(json)
    }
}
