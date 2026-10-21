# 华为隐私声明托管（推荐）配置说明

本应用目标上架中国大陆 HarmonyOS 应用市场，分发设备含**平板 / PC/2in1**，符合华为「标准化隐私声明托管」适用范围。

官方说明：

- [配置隐私声明](https://developer.huawei.com/consumer/cn/doc/doccenter-submission/agc-help-release-app-privacy-state-0000002278878296)
- [隐私声明概述](https://developer.huawei.com/consumer/cn/doc/doccenter-submission/agc-help-privacy-policy-overview-0000002316881805)
- [FAQ · 管理隐私声明](https://developer.huawei.com/consumer/cn/doc/App/agc-help-privacy-policy-faq-0000002342315628)

## 为什么优先用华为托管

| 方式 | 说明 |
|------|------|
| **隐私托管（推荐）** | 在 HarmonyOS 开发者中心按模板生成政策；**系统提供首启同意弹窗**；审核友好。应用类推荐使用，非强制。 |
| 自定义隐私政策 URL | 需自备可公网访问的 https 页面。本仓库为**私有**，GitHub raw/Pages **不能**直接给审核员访问。 |

**个人开发者可用托管**，无需公司主体。托管生成后的政策 URL 由华为侧提供，**无需再单独建公开 GitHub 法律仓库**。

## 控制台操作（你需要登录开发者账号完成）

1. 打开 [HarmonyOS 开发者中心 / AppGallery Connect](https://developer.huawei.com/) → 应用管理 → 选中 **艺工坊·修图**（包名 `io.github.mxjinjing.artforge.photo`）。
2. 进入 **应用上架 → 协议服务**（或版本信息中的「隐私声明」区域）。
3. **新建协议** → 类型选 **隐私政策** → 模板选 **HarmonyOS 应用** → 默认语言 **简体中文**。
4. 按下方「模板填写对照」如实勾选与填写。
5. 预览无误后 **生成协议**。
6. 在待发布版本的「隐私声明」中选择 **隐私托管**，并关联刚生成的协议。
7. 将生成后的 **隐私政策 https 链接** 记下来（已填入 `links.rs` 的 `PRIVACY_STATEMENT` 与 README「帮助菜单」表格；若重新生成需同步更新），并按需写入 debug `module.json5` metadata（见下文）。

> 托管启用后：**不要**再在应用内自行实现首启隐私弹窗，否则可能因重复弹窗被拒。本仓库代码使用 `privacyManager` **查询签署状态并延后业务初始化**，不自绘系统级同意窗。

## 模板填写对照（与当前代码一致）

依据 `entry/src/main/module.json5` 与现有 ArkTS 实现：

### 导语 / 简介（示例）

> 「艺工坊·修图」（ArtForge Photo）是一款面向 HarmonyOS 平板与 PC/2in1 的本地图像编辑应用，支持图层编辑、导入导出等。由个人开发者 MXJinjing 提供，以设备本地处理为主。

### 收集用户数据说明

建议如实体现「以本地处理为主」：

- **一般做法**：若模板允许声明「不收集」或仅描述本地文件由用户主动选择处理，请按控制台选项勾选与文案 `docs/privacy-policy.md` 第二节一致。
- **不要勾选**当前代码未使用的：账号、位置、通讯录、短信、OAID 统计、剪贴板画像等。
- 用户主动导入的图片：业务场景写「图片编辑 / 打开与保存文件」；时机为「用户主动选择文件或图库图片时」。

### 设备权限调用

| 权限 | 作用说明 |
|------|----------|
| 打印（PRINT） | 用户主动打印编辑结果时调用系统打印 |
| 网络（INTERNET） | 打开外部链接等必要联网；非静默分析 |

图库/文件：说明通过系统选择器由用户授权，非后台读相册。

### 第三方 SDK

填写：**无**（或留空 / 不增加 SDK 条目）。当前未接入统计、广告、推送、第三方登录 SDK。

### 与第三方共享

填写：不向第三方出售或共享个人画像类信息；系统打印/文件选择遵循系统规则。

### 个性化推荐 / 广告营销

不涉及则不要增加说明。

### 未成年人

声明不专门面向不满 14 周岁儿童；不主动收集儿童信息。若模板强制要求儿童政策 URL，可用托管自定义章节说明「不适用专门儿童产品」，或按审核要求补充简短儿童说明页。

### 信息存储

- 期限：实现处理目的所必要的最短时间  
- 位置：用户设备本地  

### 如何联系我们

- 身份：个人开发者  
- 邮箱：**请替换为你的长期有效联系邮箱**（仓库内仍为占位符 `请替换为你的联系邮箱`）

### 生效日期

填写生成当日，例如 **2026-10-20**（之后每次修订更新）。

### 开发者自定义章节（建议）

可粘贴 `docs/privacy-policy.md` 中「不主动收集的信息」「无账号体系」等摘要，每章不超过 3000 字，最多 5 章。

## 应用内行为（本仓库已实现）

- `entry/src/main/ets/privacy/PrivacyConsent.ets`：通过 AppGallery Kit `privacyManager` 查询签署结果，并订阅 `usual.event.PRIVACY_STATE_CHANGED`；提供托管链接查询。
- `pages/Index.ets`：仅在用户同意（或本地未配置托管时的开发回退）后执行 `native.init`、启动打印适配等业务；加载页提供「查看隐私声明」与长按标题入口。
- `entryability/EntryAbility.ets`：隐私未同意前不调用 `native.active(true)`。
- **禁止**自绘首启隐私 CustomDialog（与系统托管弹窗重复会被拒审）。

### 帮助菜单入口

鸿蒙构建的「帮助」菜单提供两个联网入口：

- **隐私政策**：直接打开华为托管生成的协议页 `https://agreement-drcn.hispace.dbankcloud.cn/index.html?lang=zh&agreementId=2066016208368459456`（Rust 侧 `links::PRIVACY_STATEMENT`）。
- **报告问题…**：打开本应用的问题反馈仓库 `https://gitcode.com/MXJinjing/artforge-photo-report`（Rust 侧 `links::ISSUES`，复用上游 `help.reportIssue` 菜单项与样式）。

两者都是普通 https 链接，由 ArkTS `pages/Index.ets` 用 `ohos.want.action.viewData` + `entity.system.browsable` 交给系统浏览器打开。

不再提供 **用户协议** 入口：应用无账号体系、无云同步、无第三方登录，当前仅在用户主动点击上述链接时产生网络请求。首启同意仍由系统托管弹窗负责（`privacyManager`），应用内不自绘弹窗。

> `artforge-privacy://statement|agreement` 伪协议通道（`PrivacyConsent.openHostedPrivacyScheme`）保留备用：Rust 侧改为直链后不再下发该 scheme，加载页的「查看隐私声明」仍直接调用 `openHostedPrivacyStatement` 走托管查询。


### 上架前 debug 模拟弹窗（可选）

未上架时，可在 **debug** 包的 `entry/src/main/module.json5` 的 `module.metadata` 中增加（URL 须为 https，且为你已生成的托管链接或临时可访问页）：

```json5
"metadata": [
  { "name": "appgallery_privacy_hosted", "value": "1" },
  { "name": "appgallery_privacy_link_privacy_statement", "value": "https://你的托管或临时隐私政策URL" }
]
```

正式发布包关联托管后，以云端配置为准；勿长期保留错误测试 URL。

## 备选：自定义隐私政策 URL

若你选择「自定义隐私政策」而非托管：

1. 需要可公网访问的 https 页面（可用 `docs/privacy-policy.html` 内容）。
2. **不要**依赖本私有仓库的 raw 链接。
3. 此时才需要自备托管（例如另建**仅含法律文档**的公开仓 / 个人站点）；并应改用应用内自绘同意弹窗，同时关闭华为托管以免双弹窗。

当前推荐路径仍是：**个人开发者 + 华为隐私托管 + 本仓库门控代码**。

## 你还需要手动完成的事项

1. 在 AGC 按本文填写并生成隐私政策，关联版本。  
2. 把真实联系邮箱写入 AGC 与 `docs/privacy-policy.md` / `.html` / 字符串资源。  
3. 若 AGC 重新生成了隐私政策导致协议页 URL 变化，同步更新 `upstream/photocraft/crates/ui-egui/src/links.rs` 的 `PRIVACY_STATEMENT` 与 README「帮助菜单」表格。  
4. （可选）配置 debug metadata 做真机弹窗联调。
