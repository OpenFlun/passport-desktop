//! 定义错误转换 trait,用于将 Windows KeyCredentialStatus 转换为 napi::Error;
use windows::Security::Credentials::KeyCredentialStatus;

/// 将 Windows 凭据状态转换为 napi::Error 的抽象接口;
pub trait PassportError {
    /// 用户取消操作时使用的错误;
    fn user_cancelled() -> Self;
    /// 未找到凭据时使用的错误（通常需要先设置 PIN）;
    fn not_found() -> Self;
    /// 用户偏好使用密码而非生物识别时使用的错误;
    fn user_prefers_password() -> Self;
    /// 凭据已存在时使用的错误;
    fn credential_already_exists() -> Self;
    /// 安全设备锁定时使用的错误;
    fn security_device_locked() -> Self;
    /// 未知状态码时使用的错误;
    fn unknown(status: KeyCredentialStatus) -> Self;

    /// 根据状态码自动选择对应的错误构造方法;
    fn from_credential_status(status: KeyCredentialStatus) -> Self;
}

impl PassportError for napi::Error {
    fn user_cancelled() -> Self {
        napi::Error::new(napi::Status::GenericFailure, "用户取消了操作")
    }

    fn not_found() -> Self {
        napi::Error::new(
            napi::Status::GenericFailure,
            "用户需要在创建 Passport 密钥之前设置 PIN 或生物识别手势",
        )
    }

    fn user_prefers_password() -> Self {
        napi::Error::new(napi::Status::GenericFailure, "用户更喜欢使用密码")
    }

    fn credential_already_exists() -> Self {
        napi::Error::new(napi::Status::GenericFailure, "凭据已存在")
    }

    fn security_device_locked() -> Self {
        napi::Error::new(napi::Status::GenericFailure, "安全设备已锁定")
    }

    fn unknown(status: KeyCredentialStatus) -> Self {
        napi::Error::new(
            napi::Status::GenericFailure,
            format!("发生未知错误;状态码: {}", status.0),
        )
    }

    /// 将 `KeyCredentialStatus` 映射为对应的 `napi::Error`;
    /// 注意：`Success` 状态本不应在此处出现,若出现则视为异常;
    fn from_credential_status(status: KeyCredentialStatus) -> Self {
        match status {
            KeyCredentialStatus::Success => napi::Error::new(
                napi::Status::GenericFailure,
                "收到成功状态,但此处期望错误状态（可能调用时机不当）",
            ),
            KeyCredentialStatus::UserCanceled => Self::user_cancelled(),
            KeyCredentialStatus::NotFound => Self::not_found(),
            KeyCredentialStatus::UserPrefersPassword => Self::user_prefers_password(),
            KeyCredentialStatus::CredentialAlreadyExists => Self::credential_already_exists(),
            KeyCredentialStatus::SecurityDeviceLocked => Self::security_device_locked(),
            s => Self::unknown(s),
        }
    }
}