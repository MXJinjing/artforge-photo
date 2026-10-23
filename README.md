# 艺工坊·修图（ArtForge Photo）

[![ArkTS](https://img.shields.io/badge/UI-ArkTS%20%2F%20ETS-blue)](https://developer.huawei.com/consumer/cn/arkts/)
[![Rust](https://img.shields.io/badge/native-Rust-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/HarmonyOS-native-blue)](https://www.harmonyos.com/)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-black)](LICENSE)
[![Status](https://img.shields.io/badge/status-private%20listing%20fork-lightgrey)](https://github.com/MXJinjing/artforge-photo)

**艺工坊·修图**（英文名 **ArtForge Photo**）是面向 HarmonyOS 的分层图像编辑应用，用于华为应用市场上架准备。

本仓库为 **私有上架分支**，基于开源鸿蒙适配 [MXJinjing/photocraft-hmos](https://github.com/MXJinjing/photocraft-hmos) 派生；后者又源自上游桌面 / Web 项目 [storytold/photocraft](https://github.com/storytold/photocraft)（PhotoCraft）。

| 项 | 值 |
|---|---|
| 中文名 | 艺工坊·修图 |
| 英文名 | ArtForge Photo |
| bundleName | `io.github.mxjinjing.artforge.photo` |
| 许可 | MIT OR Apache-2.0（延续上游双许可） |

> [!NOTE]
> 本仓库为个人上架用私有 fork，**与上游 PhotoCraft / ArtCraft 团队无关**，未获其官方支持或背书。功能与稳定性仍在完善中，**请谨慎用于实际生产场景**，编辑过程中请及时保存。

Adobe、Photoshop 等是 Adobe Inc. 的商标。本应用为独立作品，与 Adobe 无关、未获其赞助或背书；文中名称仅用于说明兼容的工作流程。

---

## 与上游的关系

- 开源鸿蒙适配源：[`MXJinjing/photocraft-hmos`](https://github.com/MXJinjing/photocraft-hmos)
- 上游 PhotoCraft（桌面 / Web）：[`storytold/photocraft`](https://github.com/storytold/photocraft)
- 本仓库保留上游与鸿蒙适配的许可证与版权声明（见 `LICENSE`、`LICENSE-MIT`、`LICENSE-APACHE`）
- 内部原生库标识（如 `libphotocraft.so`、Rust crate 名）为技术实现细节，未随上架品牌一并重命名

完整桌面能力说明请参阅上游项目与介绍页：[getartcraft.com/apps/photocraft](https://getartcraft.com/apps/photocraft)

---

## 鸿蒙版能力摘要

相对上游桌面 / Web 版，鸿蒙宿主额外提供：

- **触屏优化** — 手指手势、双指平移 / 缩放
- **华为 M-Pencil 适配** — 压感、笔身手势（效果因机型与笔型号而异）
- **系统色彩管理** — SDR Display P3（必要时回退 sRGB）；预览仍为 8-bit

其余图层、PSD、调整、选区等能力以设备上实际表现为准。

---

## 构建说明

本仓库面向应用市场上架打包，默认不提供公开 Releases 侧载包。构建需安装 HarmonyOS 开发者套件，并在 `scripts/dev.local.env` 中按需配置 SDK、hvigor 和 hdc 路径。

### 编译 Rust 原生库

```bash
scripts/build_native.sh
```

默认使用 **debug** profile，并按 debug 模式编译；可显式选择 release：

```bash
PHOTOCRAFT_RUST_PROFILE=release PHOTOCRAFT_HAP_BUILD_MODE=release scripts/build_native.sh
```

`PHOTOCRAFT_RUST_PROFILE` 控制 Rust 库的 Cargo profile；`PHOTOCRAFT_HAP_BUILD_MODE` 控制应用的构建标记和调试菜单。未设置 HAP 模式时，它跟随 Rust profile。两项可分别设置，例如 debug HAP 使用 release profile 的 Rust 库：

```bash
PHOTOCRAFT_RUST_PROFILE=release PHOTOCRAFT_HAP_BUILD_MODE=debug scripts/build_native.sh
```

### 构建 HAP

- **Debug HAP：** `scripts/dev.sh build` 构建并校验 unsigned HAP；`scripts/dev.sh run` 还会安装并启动应用，要求连接且仅连接一台设备或模拟器，并配置本地 debug 签名。
- **Release 上架包：** `scripts/build_release.sh --version 0.12.0.2` 构建并校验签名后的 HAP 和 App Pack。需先配置本地 `build-profile.json5` 中名为 `release` 的签名项；签名材料不会提交到仓库。
- **直接使用 DevEco Studio：** IDE 不会自动重编 Rust 库。构建 HAP 前，先用上面的命令编译与目标模式匹配的 Rust 库；当前 CMake 默认链接 release profile 的库。通常建议用 `scripts/dev.sh build` 或 `scripts/build_release.sh` 完成整套构建。

建议系统版本为 **HarmonyOS 6.0** 及以上；原生库为 **arm64-v8a**。


---

## 首次启动隐私提示（自有网站）

首次启动显示应用自己的隐私弹窗，提供《隐私政策》链接、“同意并继续”和“不同意并退出”。点击政策后在同一弹窗内显示网站页面，并提供“在浏览器中打开”；阅读时隐藏同意和拒绝按钮，返回仍需明确选择；仅点击同意且记录成功写入本地后才初始化原生编辑器、文档目录与打印业务。

`PrivacyConsent.ets` 使用本地 Preferences 保存当前政策版本，不依赖华为隐私托管。读取或写入失败时不自动放行。涉及信息处理方式的政策更新时递增 `PRIVACY_CONSENT_VERSION`，旧同意记录随即失效。政策阅读组件仅在用户主动点击时加载网站，未启用 JavaScript 或文件访问。

### 帮助菜单

「帮助」菜单下的两个联网入口：

| 菜单项 | 打开的地址 |
| --- | --- |
| **隐私政策** | `https://www.a999.cloud/statics/artforge-photo-privacy-policy.html` |
| **报告问题…** | 本应用的问题反馈仓库 `https://gitcode.com/MXJinjing/artforge-photo-report` |

两者都通过系统浏览器打开（`ohos.want.action.viewData` + `entity.system.browsable`）。**没有「用户协议」入口**：本应用无账号体系，图片编辑在本地完成；应用自身不向远程服务器上传用户信息。


### 开发与侧载环境中的隐私确认

调试、侧载和正式包使用同一套首启提示。清除应用数据后可重新验证。`module.json5` 已移除华为托管 metadata；上架后台使用自定义隐私政策 URL，详见 [隐私配置与验收说明](docs/huawei-privacy-hosting.md)。

## 许可证

双许可 **MIT OR Apache-2.0**，与上游 PhotoCraft / photocraft-hmos 一致。详见根目录 `LICENSE`。

Copyright 声明中的 HarmonyOS 移植版权仍归 MXJinjing；上游 PhotoCraft 版权归 ArtCraft Team 与贡献者。
