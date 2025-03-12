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
        impl $crate::traits::Inquire<$carry_on_name> for $enum_name {
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
