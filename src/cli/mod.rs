trait Inquire
where
    Self: Sized,
{
    fn inquire() -> Option<Self>;
}

/// Macro to implement `Inquire` for enums that derive `EnumIter`
macro_rules! impl_inquire_selection {
    ($enum_name:ident) => {
        impl crate::cli::Inquire for $enum_name {
            fn inquire() -> Option<$enum_name> {
                let options: Vec<$enum_name> = $enum_name::iter().collect();

                Select::new("Choose subcommand:", options)
                    .with_formatter(&|a| format!("{a}"))
                    .prompt()
                    .ok()
            }
        }
    };
}

trait Handle
where
    Self: Sized + Inquire,
{
    fn handle(&self);

    fn handle_optn(action: &Option<Self>) {
        if let Some(action) = action {
            action.handle();
        } else {
            let result = Self::inquire();
            if let Some(action) = result {
                action.handle();
            }
        };
    }
}

mod handlers;
pub use handlers::Cli;
