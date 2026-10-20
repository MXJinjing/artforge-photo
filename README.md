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
| bundleName | `wang.jinjing.artforge.photo` |
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

本仓库面向应用市场上架打包，默认不提供公开 Releases 侧载包。具备 HarmonyOS 开发者套件时，可从源码构建后使用 `hdc` 安装（构建流程与 `photocraft-hmos` 一致，参见该仓库文档与 `scripts/`）。

建议系统版本为 **HarmonyOS 6.0** 及以上；原生库为 **arm64-v8a**。

---

## 许可证

双许可 **MIT OR Apache-2.0**，与上游 PhotoCraft / photocraft-hmos 一致。详见根目录 `LICENSE`。

Copyright 声明中的 HarmonyOS 移植版权仍归 MXJinjing；上游 PhotoCraft 版权归 ArtCraft Team 与贡献者。
