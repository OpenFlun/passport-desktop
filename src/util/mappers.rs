// 此文件定义 MapNapiError trait,用于将任意 Error 类型映射为 napi::Error,简化错误转换;
use std::error::Error;

pub trait MapNapiError<T> {
    fn map_napi_error(self) -> napi::Result<T>;
}

impl<T, U: Error> MapNapiError<T> for Result<T, U> {
    fn map_napi_error(self) -> napi::Result<T> {
        self.map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}