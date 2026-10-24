# Changelog
## [1.0.8] - 2026-10-24 21:29

### 升级
- tokio 升级到 1.54.1,修复 1.54.0 的竞态条件漏洞;
- futures 及其子 crate 升级到 0.3.34;
- napi 生态升级:napi 3.15.0、napi-derive 3.6.13、napi-build 2.6.0、napi-sys 3.4.1;
- 连带更新 convert_case 0.12.0、syn 3.0.9(与 syn 2.0.119 共存)、ctor 1.0.13;
- 同批更新 bitflags 2.13.2、cfg-if 1.0.5、unicode-ident 1.0.27;

### 修复
- Cargo.toml 增加 [lints.rust] linker_messages = "allow",消除链接成功时的误报警告;

### 文档
- 指南第 5 节改为直接将二进制复制到平台包目录,不再经项目内 npm 目录中转;
- 指南构建命令改为 cmd /c 形式,避免 PowerShell 将 cargo 进度信息染红;
- 指南补充关闭 linker_messages 警告的配置说明;
- 指南章节编号修正,配置 package.json 一节编为第 7 节,后续更新流程顺延为第 8 节;

## [1.0.7] - 2026-10-24 19:47

### 优化
- 项目标点已改为以英文标点为主;
- 行宽改为以 123 为主,部分超宽表格改为了列表;

## [1.0.6] - 2026-07-26 11:47
### 优化
- 项目结构深度优化,删除大量冗余代码;