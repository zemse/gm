#[derive(thiserror::Error, Debug)]
pub enum RatatuiExtraError {
    #[error("Label at cursor {cursor} not available. Available labels: {available:?}")]
    FormLabelNotAvailable {
        cursor: usize,
        available: Vec<String>,
    },

    #[error("Select list is not set.")]
    SelectListNotSet,

    #[error("Select list is not set. (idx: {idx}, list_len: {list_len})")]
    SelectItemNotFound { idx: usize, list_len: usize },
}
