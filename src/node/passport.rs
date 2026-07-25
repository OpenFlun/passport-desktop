//! Passport 模块提供对 Windows Hello (Passkey) 的原生支持;包含账户管理、签名、验证等功能;
use crate::node::key_creation_option::KeyCreationOption;
use crate::node::public_key_encoding::PublicKeyEncoding;
use crate::node::verification_result::VerificationResult;
use crate::util::errors::PassportError;
use crate::util::mappers::MapNapiError;
use crate::util::traits::IntoWinBuffer;
use crate::win::passport::{create_passport_key, get_passport_account, get_passport_account_sync};
use napi::bindgen_prelude::Buffer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::core::{HSTRING, PWSTR};
use windows::Security::Credentials::UI::UserConsentVerifier;
use windows::Security::Credentials::{KeyCredentialManager, KeyCredentialStatus};
use windows::Security::Cryptography::Core::CryptographicPublicKeyBlobType;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, VK_MENU, KEYEVENTF_KEYUP, KEYBD_EVENT_FLAGS
};

/// Passport 实例,绑定一个唯一的账户 ID,管理 Windows Hello 凭证;
#[napi]
pub struct Passport {
    account_id: String,
    account_exists: Arc<Mutex<bool>>,
}

#[napi]
impl Passport {
    /// 构造函数:创建新的 Passport 实例,立即检查账户是否存在;
    #[napi(constructor)]
    pub fn new(account_id: String) -> napi::Result<Self> {
        let exists = Self::account_with_id_exists(account_id.clone())?;
        Ok(Self {
            account_id,
            account_exists: Arc::new(Mutex::new(exists)),
        })
    }

    /// 创建 Windows Hello 账户（注册 Passkey）,如果账户已存在且 `creation_option` 不是 `FailIfExists`,则替换;
    #[napi]
    pub fn create_account(
        &self,
        creation_option: Option<KeyCreationOption>,
    ) -> napi::Result<()> {
        let stop = start_window_activation_polling();
        let result = pollster::block_on(async {
            let account_id = self.account_id.clone();
            let account_exists = self.account_exists.clone();
            let create_result = create_passport_key(
                &account_id,
                creation_option.map(|o| o.into()),
            )
            .await
            .map_napi_error()?;
            match create_result {
                KeyCredentialStatus::Success => {
                    *account_exists.lock().unwrap() = true;
                    Ok(())
                }
                s => Err(napi::Error::from_credential_status(s)),
            }
        });
        if result.is_ok() {
            thread::sleep(Duration::from_millis(150));
            activate_credential_window();
        }
        stop.store(true, Ordering::Relaxed);
        result
    }

    /// 使用私钥对挑战进行签名（登录验证）;会弹出 Windows Hello 对话框要求用户验证身份;
    #[napi]
    pub fn sign(&self, challenge: Buffer) -> napi::Result<Buffer> {
        let stop = start_window_activation_polling();
        let result = pollster::block_on(async {
            let account_id = self.account_id.clone();
            let account_exists = self.account_exists.clone();
            if !*account_exists.lock().unwrap() {
                return Err(napi::Error::from_reason("账户不存在"));
            }
            let credential = get_passport_account(&account_id)
                .await
                .map_napi_error()?;
            let status = credential.Status().map_napi_error()?;
            if status != KeyCredentialStatus::Success {
                return Err(napi::Error::from_credential_status(status));
            }
            let credential = credential.Credential().map_napi_error()?;
            let res = credential
                .RequestSignAsync(&challenge.into_win_buffer().map_napi_error()?)
                .map_napi_error()?
                .await
                .map_napi_error()?;
            let status = res.Status().map_napi_error()?;
            if status != KeyCredentialStatus::Success {
                return Err(napi::Error::from_credential_status(status));
            }
            Buffer::from_win_buffer(res.Result().map_napi_error()?).map_napi_error()
        });
        if result.is_ok() {
            thread::sleep(Duration::from_millis(150));
            activate_credential_window();
        }
        stop.store(true, Ordering::Relaxed);
        result
    }

    /// 删除 Windows Hello 账户（移除 Passkey）;
    #[napi]
    pub fn delete_account(&self) -> napi::Result<()> {
        if !*self.account_exists.lock().unwrap() {
            return Err(napi::Error::from_reason("账户不存在"));
        }
        pollster::block_on(async {
            KeyCredentialManager::DeleteAsync(&HSTRING::from(&self.account_id))
                .map_napi_error()?
                .await
                .map_napi_error()
        })?;
        *self.account_exists.lock().unwrap() = false;
        Ok(())
    }

    /// 获取账户的公钥（用于验证签名）;
    #[napi]
    pub fn get_public_key(
        &self,
        encoding: Option<PublicKeyEncoding>,
    ) -> napi::Result<Buffer> {
        if !*self.account_exists.lock().unwrap() {
            return Err(napi::Error::from_reason("账户不存在"));
        }
        pollster::block_on(async {
            let credential = get_passport_account(&self.account_id)
                .await
                .map_napi_error()?;
            let status = credential.Status().map_napi_error()?;
            if status != KeyCredentialStatus::Success {
                return Err(napi::Error::from_credential_status(status));
            }
            let credential = credential.Credential().map_napi_error()?;
            let res = credential
                .RetrievePublicKeyWithBlobType(
                    encoding
                        .map(|e| e.into())
                        .unwrap_or(CryptographicPublicKeyBlobType::Pkcs1RsaPublicKey),
                )
                .map_napi_error()?;
            Buffer::from_win_buffer(res).map_napi_error()
        })
    }

    /// 账户是否存在于 Windows 凭据管理器中;
    #[napi(getter)]
    pub fn account_exists(&self) -> bool {
        *self.account_exists.lock().unwrap()
    }

    /// 检测当前平台是否支持 Windows Hello;
    #[napi]
    pub fn available() -> napi::Result<bool> {
        pollster::block_on(async {
            KeyCredentialManager::IsSupportedAsync()
                .map_napi_error()?
                .await
                .map_napi_error()
        })
    }

    /// 静态方法,检查指定 ID 的账户是否存在;
    #[napi]
    pub fn account_with_id_exists(id: String) -> napi::Result<bool> {
        let status = get_passport_account_sync(&id)
            .map_napi_error()?
            .Status()
            .map_napi_error()?;
        match status {
            KeyCredentialStatus::Success => Ok(true),
            KeyCredentialStatus::NotFound => Ok(false),
            s => Err(napi::Error::from_credential_status(s)),
        }
    }

    /// 主动请求用户验证（弹出对话框）;
    #[napi]
    pub fn request_verification(message: String) -> napi::Result<VerificationResult> {
        VerificationResult::try_from(
            pollster::block_on(async {
                UserConsentVerifier::RequestVerificationAsync(&HSTRING::from(message))
                    .map_napi_error()?
                    .await
                    .map_napi_error()
            })?,
        )
    }
}

/// 启动一个后台线程,每隔 200ms 查找并激活 `CredentialUIBroker.exe` 窗口,
/// 同时发送 Alt 键以提升窗口层级;返回一个 `AtomicBool` 标志,设为 `true` 即可停止轮询;
fn start_window_activation_polling() -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    thread::spawn(move || {
        while !stop_clone.load(Ordering::Relaxed) {
            activate_credential_window();
            thread::sleep(Duration::from_millis(200));
        }
    });
    stop
}

/// 单次尝试激活 Windows Hello 窗口, 通过枚举所有顶层窗口,匹配进程名 `CredentialUIBroker.exe`;
fn activate_credential_window() {
    unsafe {
        let mut found = false;
        let _ = EnumWindows(
            Some(enum_window_proc),
            LPARAM(&mut found as *mut _ as isize),
        );
    }
}

/// `EnumWindows` 回调：查找目标进程并激活窗口;
unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    unsafe {
        let found = &mut *(lparam.0 as *mut bool);
        if *found {
            return windows::core::BOOL(0);
        }

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));
        if process_id == 0 {
            return windows::core::BOOL(1);
        }

        if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
            let mut buffer = [0u16; 260];
            let mut size = buffer.len() as u32;
            let result = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0),
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            );
            if result.is_ok() {
                let name = String::from_utf16_lossy(&buffer[..size as usize]);
                if name.contains("CredentialUIBroker.exe") {
                    let _ = SetForegroundWindow(hwnd);
                    let vk_menu = VK_MENU.0 as u8;
                    keybd_event(vk_menu, 0, KEYBD_EVENT_FLAGS(0), 0);
                    keybd_event(vk_menu, 0, KEYEVENTF_KEYUP, 0);
                    *found = true;
                    return windows::core::BOOL(0);
                }
            }
        }
        windows::core::BOOL(1)
    }
}