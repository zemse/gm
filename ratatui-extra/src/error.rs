#[derive(thiserror::Error, Debug)]
pub enum RatatuiExtraError {
    #[error("Label at cursor {cursor} not available. Available labels: {available:?}")]
    FormLabelNotAvailable {
        cursor: usize,
        available: Vec<String>,
    },
}
