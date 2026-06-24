use std::cmp::Ordering;

/// 정렬 비교: sort_order 오름차순, 동률이면 id 문자열 오름차순.
/// NaN 등 비교 불가는 Equal 로 처리해 패닉을 피한다.
pub fn cmp_sort(a_order: f64, a_id: &str, b_order: f64, b_id: &str) -> Ordering {
    a_order
        .partial_cmp(&b_order)
        .unwrap_or(Ordering::Equal)
        .then_with(|| a_id.cmp(b_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_by_sort_then_id() {
        assert_eq!(cmp_sort(1.0, "z", 2.0, "a"), Ordering::Less);
        assert_eq!(cmp_sort(2.0, "a", 1.0, "z"), Ordering::Greater);
        // 동률이면 id 로 안정화
        assert_eq!(cmp_sort(1.0, "a", 1.0, "b"), Ordering::Less);
        assert_eq!(cmp_sort(1.0, "b", 1.0, "b"), Ordering::Equal);
    }

    #[test]
    fn nan_does_not_panic() {
        let _ = cmp_sort(f64::NAN, "a", 1.0, "b");
        assert_eq!(cmp_sort(f64::NAN, "a", 1.0, "b"), Ordering::Less); // NAN→Equal then id "a"<"b"
    }
}
