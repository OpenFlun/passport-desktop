// 此文件定义宏 check_account_exists!,用于在方法中快速检查帐户是否存在,若不存在则返回错误;
#[macro_export]
macro_rules! check_account_exists {
    ($account_exists: expr) => {
        if !$account_exists.lock().unwrap().clone() {
            return Err(napi::Error::new(
                napi::Status::GenericFailure,
                "Passport 帐户不存在",
            ));
        }
    };
}