const SERVICE: &str = "com.chaejinseong.ai-os-dashboard";
pub fn get(account: &str) -> Result<Option<String>, String> {
    match keyring::Entry::new(SERVICE, account)
        .map_err(|_| "Keychain 접근 실패")?
        .get_password()
    {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Keychain 읽기 실패".into()),
    }
}
pub fn set(account: &str, value: &str) -> Result<(), String> {
    keyring::Entry::new(SERVICE, account)
        .map_err(|_| "Keychain 접근 실패")?
        .set_password(value)
        .map_err(|_| "Keychain 저장 실패".into())
}
pub fn delete(account: &str) -> Result<(), String> {
    match keyring::Entry::new(SERVICE, account)
        .map_err(|_| "Keychain 접근 실패")?
        .delete_credential()
    {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Keychain 삭제 실패".into()),
    }
}
