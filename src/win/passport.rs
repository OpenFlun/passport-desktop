//! 底层 Windows KeyCredentialManager 异步/同步调用封装;
use windows::core::HSTRING;
use windows::Security::Credentials::{
    KeyCredentialCreationOption, KeyCredentialManager, KeyCredentialRetrievalResult,
    KeyCredentialStatus,
};

/// 异步创建 Passkey 密钥;
pub async fn create_passport_key(
    account_id: &String,
    create_option: Option<KeyCredentialCreationOption>,
) -> windows::core::Result<KeyCredentialStatus> {
    KeyCredentialManager::RequestCreateAsync(
        &HSTRING::from(account_id),
        create_option.unwrap_or(KeyCredentialCreationOption::ReplaceExisting),
    )?
    .await?
    .Status()
}

/// 异步获取账户凭据;
pub async fn get_passport_account(
    account_id: &String,
) -> windows::core::Result<KeyCredentialRetrievalResult> {
    KeyCredentialManager::OpenAsync(&HSTRING::from(account_id))?.await
}

/// 同步版本（阻塞当前线程）,用于无法 async 的上下文;
pub fn get_passport_account_sync(
    account_id: &String,
) -> windows::core::Result<KeyCredentialRetrievalResult> {
    let operation = KeyCredentialManager::OpenAsync(&HSTRING::from(account_id))?;
    pollster::block_on(operation)
}