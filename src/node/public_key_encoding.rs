//! 定义公钥编码格式枚举,用于指定导出 Passport 公钥时的序列化格式;
//! 在调用 `Passport::get_public_key` 时,可以通过该枚举选择不同的编码方式,以适应不同的互操作性需求;

use windows::Security::Cryptography::Core::CryptographicPublicKeyBlobType;
/// 导出公钥时使用的编码格式;默认使用 `Pkcs1RsaPublicKey`;
#[napi]
pub enum PublicKeyEncoding {
    X509SubjectPublicKeyInfo,
    Pkcs1RsaPublicKey,
    BCryptPublicKey,
    Capi1PublicKey,
    BCryptEccFullPublicKey
}

/// 自动转换为 Windows API 所需的类型,无需手动转换;
impl From<PublicKeyEncoding> for CryptographicPublicKeyBlobType {
    fn from(value: PublicKeyEncoding) -> Self {
        match value {
            PublicKeyEncoding::X509SubjectPublicKeyInfo => {
                CryptographicPublicKeyBlobType::X509SubjectPublicKeyInfo
            }
            PublicKeyEncoding::Pkcs1RsaPublicKey => {
                CryptographicPublicKeyBlobType::Pkcs1RsaPublicKey
            }
            PublicKeyEncoding::BCryptPublicKey => CryptographicPublicKeyBlobType::BCryptPublicKey,
            PublicKeyEncoding::Capi1PublicKey => CryptographicPublicKeyBlobType::Capi1PublicKey,
            PublicKeyEncoding::BCryptEccFullPublicKey => {
                CryptographicPublicKeyBlobType::BCryptEccFullPublicKey
            }
        }
    }
}