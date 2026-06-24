use serde::{Serialize, Serializer};

/// 앱 단일 에러 타입. Tauri 커맨드 경계에서 문자열로 직렬화되어 프론트로 전달.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("대상을 찾을 수 없음")]
    NotFound,
    #[error("잘못된 요청: {0}")]
    Invalid(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
