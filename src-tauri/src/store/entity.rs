use serde::de::DeserializeOwned;
use serde::Serialize;

/// 파일 저장 가능한 엔티티. collection()은 디스크 하위 디렉터리명.
pub trait Entity: Serialize + DeserializeOwned + Clone {
    fn collection() -> &'static str;
    fn id(&self) -> &str;
    fn updated_at(&self) -> &str;
}
