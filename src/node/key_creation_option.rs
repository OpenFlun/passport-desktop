//! 定义密钥创建选项枚举,用于在创建 Passport 密钥时控制冲突处理行为;
//! 该枚举映射到 Windows 的 `KeyCredentialCreationOption`,方便在 N-API 层使用;

use windows::Security::Credentials::KeyCredentialCreationOption;
/// 创建 Passport 密钥时的行为选项:如果账户已存在,是覆盖还是报错;
#[napi]
pub enum KeyCreationOption {
    ReplaceExisting,
    FailIfExists,
}

/// 自动转换为 Windows API 所需的类型;
impl From<KeyCreationOption> for KeyCredentialCreationOption {
    fn from(value: KeyCreationOption) -> Self {
        match value {
            KeyCreationOption::ReplaceExisting => Self::ReplaceExisting,
            KeyCreationOption::FailIfExists => Self::FailIfExists,
        }
    }
}