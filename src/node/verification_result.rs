//! 定义 Windows Hello 用户验证结果枚举,用于表示验证操作的各种可能结果;
use windows::Security::Credentials::UI::UserConsentVerificationResult;

/// 用户验证操作的结果;用于 `Passport::request_verification` 方法的返回值;
#[napi]
pub enum VerificationResult {
    /// 用户成功完成验证（如指纹、PIN 或人脸识别）;
    Verified,
    /// 设备不支持 Windows Hello（无指纹、摄像头等）;
    DeviceNotPresent,
    /// 当前用户未设置 Windows Hello;
    NotConfiguredForUser,
    /// 系统策略禁用了 Windows Hello;
    DisabledByPolicy,
    /// 设备正忙,无法处理验证请求;
    DeviceBusy,
    /// 超过最大尝试次数,验证失败;
    RetriesExhausted,
    /// 用户主动取消了验证对话框;
    Canceled,
}

/// 自动从 Windows API 结果转换,无需手动处理;
impl TryFrom<UserConsentVerificationResult> for VerificationResult {
    type Error = napi::Error;

    fn try_from(value: UserConsentVerificationResult) -> napi::Result<Self> {
        Ok(match value {
            UserConsentVerificationResult::Verified => Self::Verified,
            UserConsentVerificationResult::DeviceNotPresent => Self::DeviceNotPresent,
            UserConsentVerificationResult::NotConfiguredForUser => Self::NotConfiguredForUser,
            UserConsentVerificationResult::DisabledByPolicy => Self::DisabledByPolicy,
            UserConsentVerificationResult::DeviceBusy => Self::DeviceBusy,
            UserConsentVerificationResult::RetriesExhausted => Self::RetriesExhausted,
            UserConsentVerificationResult::Canceled => Self::Canceled,
            v => {
                return Err(napi::Error::new(
                    napi::Status::GenericFailure,
                    format!("未知的验证结果: {}", v.0),
                ))
            }
        })
    }
}