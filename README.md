# @flun/passport-desktop-win32-*-msvc

**Windows Hello 原生插件**,提供 Node.js 环境下的生物识别(指纹、人脸、PIN)认证能力;
本包是平台特定的二进制插件,分别提供 `x64` 和 `ia32` 版本;

---

## 安装

根据您的系统架构选择对应的包( @flun 只是我自己的命名空间名,如果你想将二进制脚本放进自己的包里,请自行定义修改):

```bash
# 64位系统
npm install @flun/passport-desktop-win32-x64-msvc

# 32位系统
npm install @flun/passport-desktop-win32-ia32-msvc
```

安装后即可直接使用,无需额外编译工具;

---

## 基本用法

### 1. 导入 `Passport` 类

```javascript
// CommonJS
const { Passport } = require('@flun/passport-desktop-win32-x64-msvc');

// ES Module
import { Passport } from '@flun/passport-desktop-win32-x64-msvc';
```

> **提示**:如果您的项目同时支持多架构,可动态根据 `process.arch` 加载对应包,例如:
> ```javascript
> const arch = process.arch;
> const pkgName = `@flun/passport-desktop-win32-${arch}-msvc`;
> const { Passport } = require(pkgName);
> ```

### 2. 检查 Windows Hello 是否可用

```javascript
if (!Passport.available()) {
  throw new Error('当前系统不支持 Windows Hello');
}
```

### 3. 管理账户

每个账户通过唯一的 `accountId`(字符串)标识;您需要先创建账户(注册 Passkey),然后才能进行签名操作;

```javascript
const passport = new Passport('my-user-id');

// 检查账户是否已存在
if (!passport.accountExists) {
  // 创建账户(会弹出 Windows Hello 验证对话框)
  await passport.createAccount();
  // 可选:传入 KeyCreationOption.ReplaceExisting 覆盖已有账户
}
```

### 4. 签名(登录验证)

当需要验证用户身份时,生成一个随机挑战(Buffer),调用 `sign` 方法,用户需通过 Windows Hello 验证(指纹/PIN/人脸),然后返回签名;

```javascript
import { randomBytes } from 'crypto';

const challenge = randomBytes(32);
const signature = await passport.sign(challenge);
// signature 是 Buffer,可用于后续验证
```

### 5. 验证签名

使用账户的公钥来验证签名是否正确;公钥可通过 `getPublicKey()` 获取,
默认返回 **PKCS#1 RSA DER** 格式,与 `crypto.createPublicKey` 的 'pkcs1' 类型兼容;

```javascript
import { createPublicKey, createVerify } from 'crypto';

// 获取公钥
const publicKeyDer = await passport.getPublicKey();
const publicKey = createPublicKey({
  key: publicKeyDer,
  format: 'der',
  type: 'pkcs1',
});

// 验证签名
const verifier = createVerify('SHA256');
verifier.update(challenge);
verifier.end();

const isValid = verifier.verify(publicKey, signature);
console.log(isValid); // true 或 false
```

### 6. 删除账户

```javascript
await passport.deleteAccount();
```

---

## 完整示例

```javascript
import { Passport } from '@flun/passport-desktop-win32-x64-msvc';
import { randomBytes, createPublicKey, createVerify } from 'crypto';

async function authenticate() {
  // 1. 检查可用性
  if (!Passport.available()) {
    throw new Error('Windows Hello 不可用');
  }

  // 2. 创建或获取账户
  const accountId = 'user123';
  const passport = new Passport(accountId);
  if (!passport.accountExists) {
    await passport.createAccount();
  }

  // 3. 生成挑战并签名
  const challenge = randomBytes(32);
  const signature = await passport.sign(challenge);

  // 4. 获取公钥并验证签名
  const publicKeyDer = await passport.getPublicKey();
  const publicKey = createPublicKey({
    key: publicKeyDer,
    format: 'der',
    type: 'pkcs1',
  });

  const verifier = createVerify('SHA256');
  verifier.update(challenge);
  verifier.end();
  const verified = verifier.verify(publicKey, signature);

  console.log('验证结果:', verified);
  return verified;
}
```

---

## 高阶封装(参考您项目中的 `authentication.js`)

在实际项目中,您可能会封装更高级的认证流程,如 WebAuthn 兼容的响应验证;以下是一个简化的封装函数:

```javascript
// 假设已从插件导入 Passport,并有一些辅助函数
async function verifyWithWindowsHello(credential, expectedChallenge, expectedOrigin, expectedRPID) {
  const passport = new Passport(credential.id);
  if (!passport.accountExists) {
    throw new Error('账户不存在,请先注册');
  }

  const challengeBuffer = Buffer.from(expectedChallenge);
  const signature = await passport.sign(challengeBuffer);

  // 使用存储的公钥验证签名
  const publicKey = createPublicKey({
    key: credential.publicKey,
    format: 'der',
    type: 'pkcs1',
  });
  const verifier = createVerify('SHA256');
  verifier.update(challengeBuffer);
  verifier.end();
  const verified = verifier.verify(publicKey, signature);

  if (!verified) {
    throw new Error('签名验证失败');
  }

  // 返回认证信息(此处简化)
  return {
    verified: true,
    authenticationInfo: {
      rpID: expectedRPID,
      credentialID: credential.id,
      userVerified: true,
      // ...其他字段
    },
  };
}
```

---

## API 参考

| 方法 / 属性                         | 说明                                                                                        |
| ----------------------------------- | ------------------------------------------------------------------------------------------- |
| `Passport.available()`              | 静态方法,返回 `true` 如果系统支持 Windows Hello;                                            |
| `new Passport(accountId)`           | 创建实例,自动检查账户是否存在(`accountExists` 属性);                                      |
| `passport.accountExists`            | 只读属性,`true` 表示该账户已注册;                                                           |
| `passport.createAccount([option])`  | 注册新 Passkey,`option` 可为 `KeyCreationOption.FailIfExists` 或 `ReplaceExisting`(默认); |
| `passport.sign(challenge: Buffer)`  | 对挑战签名,返回签名 `Buffer`;                                                               |
| `passport.getPublicKey([encoding])` | 获取公钥,默认 `PublicKeyEncoding.Pkcs1RsaPublicKey`(DER 格式);                            |
| `passport.deleteAccount()`          | 删除当前账户;                                                                               |
| `Passport.accountWithIdExists(id)`  | 静态方法,检查指定账户是否存在(返回 `Promise<boolean>`);                                   |

**枚举**:
- `KeyCreationOption`:`FailIfExists` / `ReplaceExisting`
- `PublicKeyEncoding`:`Pkcs1RsaPublicKey` / `BcryptPublicKey` / `CngPublicKey`

---

## 注意事项

- **平台限制**:仅在 Windows 10/11 上有效,其他平台调用会抛出异常;
- **用户交互**:`createAccount` 和 `sign` 会弹出系统对话框,需要用户确认;
- **错误处理**:如果用户取消验证,方法会抛出错误(如 `UserCanceled`),建议使用 `try...catch`;
- **公钥格式**:`getPublicKey()` 默认返回 PKCS#1 DER 格式,也可通过编码参数获取其他格式;

---

## 常见问题

**Q:为什么首次调用 `sign` 时窗口没有置前?**
A:本插件内部已优化,会在操作期间持续尝试提升窗口层级,确保体验流畅;

**Q:如何更新账户的 Passkey?**
A:调用 `createAccount(KeyCreationOption.ReplaceExisting)` 即可覆盖已有账户;

**Q:账户 ID 可以随意指定吗?**
A:可以,但建议使用与用户绑定的唯一标识(如用户 ID 或邮箱),且同一 ID 在不同应用中不共享;