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

    // LWW 불변식 잠금: now() 포맷이 고정폭이라 사전식(문자열) 정렬 == 시간 정렬이어야 한다.
    // 이 불변식이 깨지면 충돌 해소(최신 updatedAt 승리)가 잘못된다(Plan 2 의존).
    #[test]
    fn now_format_is_fixed_width_lexicographically_chronological() {
        use chrono::{NaiveDateTime, TimeZone, Utc};
        let fmt = "%Y-%m-%dT%H:%M:%S%.3fZ";
        let earlier = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap().format(fmt).to_string();
        let later = Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap().format(fmt).to_string();
        // 고정폭(24자) → 사전식 비교가 시간 비교와 일치
        assert_eq!(earlier.len(), 24);
        assert_eq!(later.len(), 24);
        assert!(earlier < later);
        // now() 가 동일 포맷이고, 그 포맷으로 다시 파싱 가능(왕복)
        let n = now();
        assert_eq!(n.len(), 24);
        let trimmed = n.trim_end_matches('Z');
        assert!(NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S%.3f").is_ok());
    }
}
