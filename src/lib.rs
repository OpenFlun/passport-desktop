// 此文件是库的根入口,声明 node、util、win 三个子模块,并引入 napi_derive 宏。
#![deny(clippy::all)]

mod node;
pub(crate) mod util;
pub(crate) mod win;

#[macro_use]
extern crate napi_derive;