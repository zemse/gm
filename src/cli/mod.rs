pub trait Inquire
where
    Self: Sized,
{
    fn inquire() -> Option<Self>;
}

/// Macro to implement `Inquire` for enums that derive `EnumIter`
#[macro_export]
macro_rules! impl_inquire_selection {
    ($enum_name:ident) => {
        impl $crate::cli::Inquire for $enum_name {
            fn inquire() -> Option<$enum_name> {
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
    Self: Sized + Inquire,
{
    fn handle(&self, carry_on: CarryOn);

    // If action value is None, call `inquire` to get the action
    fn handle_optn_inquire(action: &Option<Self>, carry_on: CarryOn) {
        if let Some(action) = action {
            action.handle(carry_on);
        } else {
            let result = Self::inquire();
            if let Some(action) = result {
                action.handle(carry_on);
            }
        };
    }
}

mod handlers;
pub use handlers::Cli;
