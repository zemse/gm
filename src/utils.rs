pub mod account;
pub mod assets;
pub mod cursor;
pub mod text;

pub trait Inquire<DefaultValue = ()>
where
    Self: Sized,
{
    fn inquire(default: &DefaultValue) -> Option<Self>;
}

/// Macro to implement `Inquire` for enums that derive `EnumIter`
#[macro_export]
macro_rules! impl_inquire_selection {
    ($enum_name:ident, $carry_on_name:tt) => {
        impl $crate::utils::Inquire<$carry_on_name> for $enum_name {
            fn inquire(_: &$carry_on_name) -> Option<$enum_name> {
                let options: Vec<$enum_name> = $enum_name::iter().collect();

                inquire::Select::new("Choose subcommand:", options)
                    .with_formatter(&|a| format!("{a}"))
                    .prompt()
                    .ok()
            }
        }
    };
}

pub trait Handle<CarryOn = ()>
where
    Self: Sized + Inquire<CarryOn>,
{
    fn handle(&self, carry_on: CarryOn);

    // If action value is None, call `inquire` to get the action
    fn handle_optn_inquire(action: &Option<Self>, carry_on: CarryOn) {
        if let Some(action) = action {
            action.handle(carry_on);
        } else {
            let result = Self::inquire(&carry_on);
            if let Some(action) = result {
                action.handle(carry_on);
            }
        };
    }
}

pub type Provider = alloy::providers::fillers::FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::Identity,
        alloy::providers::fillers::JoinFill<
            alloy::providers::fillers::GasFiller,
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::BlobGasFiller,
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::NonceFiller,
                    alloy::providers::fillers::ChainIdFiller,
                >,
            >,
        >,
    >,
    alloy::providers::RootProvider,
>;

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
    serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(
        &response.text().await?,
    ))
    .map_err(|e| crate::Error::SerdePathToError(Box::new(e)))
}
