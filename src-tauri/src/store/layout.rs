//! 순수 경로/이름 계산 — I/O 없음. 앱 메타 이름을 파일시스템 안전 이름으로 변환한다.

pub const DELIVERABLES_AREA: &str = "산출물";

const FORBIDDEN: [char; 9] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// 파일시스템 안전한 단일 경로 요소로 변환. 금지문자/제어문자 → '_', 앞뒤 공백·'.' trim.
/// 결과가 비면 fallback_id 앞 8자를 쓴다.
pub fn sanitize_component(name: &str, fallback_id: &str) -> String {
    let replaced: String = name
        .chars()
        .map(|c| if FORBIDDEN.contains(&c) || c.is_control() { '_' } else { c })
        .collect();
    let trimmed = replaced.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        let n = fallback_id.len().min(8);
        fallback_id[..n].to_string()
    } else {
        trimmed.to_string()
    }
}

/// desired(파일명 또는 폴더명)가 existing 과 충돌하면 " (2)", " (3)" … 접미.
/// 확장자가 있으면 stem 뒤에 접미를 붙인다. 비교는 대소문자 무시.
pub fn unique_name(existing: &[String], desired: &str) -> String {
    let lower: std::collections::HashSet<String> =
        existing.iter().map(|s| s.to_lowercase()).collect();
    if !lower.contains(&desired.to_lowercase()) {
        return desired.to_string();
    }
    let (stem, ext) = split_ext(desired);
    let mut n = 2;
    loop {
        let candidate = match &ext {
            Some(e) => format!("{stem} ({n}).{e}"),
            None => format!("{stem} ({n})"),
        };
        if !lower.contains(&candidate.to_lowercase()) {
            return candidate;
        }
        n += 1;
    }
}

/// "a.png" -> ("a", Some("png")), "a" -> ("a", None), ".gitignore" -> (".gitignore", None)
fn split_ext(name: &str) -> (String, Option<String>) {
    match name.rfind('.') {
        Some(i) if i > 0 && i < name.len() - 1 => {
            (name[..i].to_string(), Some(name[i + 1..].to_string()))
        }
        _ => (name.to_string(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_forbidden_and_trims() {
        assert_eq!(sanitize_component("a/b:c*?", "deadbeef"), "a_b_c__");
        assert_eq!(sanitize_component("  이름.  ", "deadbeef"), "이름");
        assert_eq!(sanitize_component("   ", "deadbeef12"), "deadbeef");
        assert_eq!(sanitize_component("정상 이름", "id"), "정상 이름");
    }

    #[test]
    fn unique_name_appends_suffix_case_insensitive() {
        let existing = vec!["설계서.png".to_string(), "설계서 (2).png".to_string()];
        assert_eq!(unique_name(&existing, "새 파일.png"), "새 파일.png");
        assert_eq!(unique_name(&existing, "설계서.png"), "설계서 (3).png");
        assert_eq!(unique_name(&existing, "SEOLGYE.PNG"), "SEOLGYE.PNG");
        let existing2 = vec!["a.PNG".to_string()];
        assert_eq!(unique_name(&existing2, "a.png"), "a (2).png");
    }
}
