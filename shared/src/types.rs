use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            message: None,
        }
    }

    pub fn with_message(data: T, message: String) -> Self {
        Self {
            data,
            message: Some(message),
        }
    }
}

