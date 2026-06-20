// OS 키체인 위임 (macOS Keychain / Windows Credential Manager).
// ⚠️ SSH 비밀번호/패스프레이즈는 여기로만 저장하고, DB에는 참조 키(secret_ref)만 남긴다.
use crate::error::{AppError, Result};
use keyring::Entry;

const SERVICE: &str = "projectmanger-ssh";

fn entry(ref_key: &str) -> Result<Entry> {
    Entry::new(SERVICE, ref_key).map_err(|e| AppError::Invalid(format!("키체인 오류: {e}")))
}

/// 비밀값 저장(있으면 덮어씀).
pub fn set(ref_key: &str, secret: &str) -> Result<()> {
    entry(ref_key)?
        .set_password(secret)
        .map_err(|e| AppError::Invalid(format!("키체인 저장 실패: {e}")))
}

/// 비밀값 조회. 없으면 None. (비밀번호 인증 자동 주입용 — 향후 사용)
#[allow(dead_code)]
pub fn get(ref_key: &str) -> Result<Option<String>> {
    match entry(ref_key)?.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Invalid(format!("키체인 조회 실패: {e}"))),
    }
}

/// 비밀값 삭제. 없어도 성공으로 취급.
pub fn delete(ref_key: &str) -> Result<()> {
    match entry(ref_key)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Invalid(format!("키체인 삭제 실패: {e}"))),
    }
}

/// 서버 연결 id로 결정적 참조 키 생성.
pub fn ref_for_server(server_id: i64) -> String {
    format!("ssh/conn-{server_id}")
}
