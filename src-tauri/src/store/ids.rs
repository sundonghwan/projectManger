/// uuid v4 문자열 생성 (엔티티 전역 id).
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 현재 시각을 ISO8601 UTC(밀리초)로. LWW 비교는 사전식 정렬=시간 정렬.
pub fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_is_unique_and_uuid_shaped() {
        let a = new_id();
        let b = new_id();
        assert_ne!(a, b);
        assert_eq!(a.len(), 36); // 8-4-4-4-12
        assert_eq!(a.matches('-').count(), 4);
    }

    #[test]
    fn now_is_iso8601_utc_millis() {
        let t = now();
        // 예: 2026-06-24T01:02:03.123Z
        assert!(t.ends_with('Z'));
        assert_eq!(t.len(), 24);
        assert_eq!(&t[10..11], "T");
    }
}
