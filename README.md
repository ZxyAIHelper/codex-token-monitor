# Codex Token Monitor

[English](./README.en.md)

Codex Token Monitor 是一个本地桌面应用，用来监控 Codex 在本机产生的 token 使用情况。它读取本地 Codex 会话日志，汇总今日、小时、会话、输入/输出、缓存输入、推理输出和工具输出风险，帮助你及时发现高消耗任务。

## 功能

- 实时仪表盘：今日 token、最近 1 小时、最近 5 小时、活跃会话数。
- 趋势视图：最近 24 小时、30 天、12 周 token 用量。
- 会话列表：按总量、输入、输出、工具调用、工具输出大小和最近更新时间排序。
- 会话详情：查看单个会话的输入内容与每轮 token 明细。
- 风险提醒：高会话用量、高小时用量、大工具输出。
- 系统托盘：后台运行、打开面板、暂停/恢复监控、退出。
- 中英文界面：应用内可切换语言。

## 数据来源

应用只读取本机 Codex 日志，不上传数据。

- Windows: `C:\Users\<user>\.codex\sessions`
- macOS/Linux: `~/.codex/sessions`

当前 MVP 以本地日志监控为主。工具输出大小只作为风险指标，不代表精确的工具 token 消耗。

## 开发环境

需要安装：

- Node.js 20 或更高版本
- Rust stable
- Tauri 2 所需的系统依赖

安装依赖：

```powershell
npm install
```

启动开发模式：

```powershell
npm run tauri dev
```

运行测试：

```powershell
npm test
```

构建前端：

```powershell
npm run build
```

构建 Windows 安装包：

```powershell
npm run tauri build
```

构建产物通常位于：

```text
src-tauri/target/release/bundle/nsis/
```

## Release 安装包

仓库包含 GitHub Actions 发布流程。推送 `v*` 标签后会在 Windows runner 上构建 NSIS 安装包，并把 `.exe` 上传到 GitHub Release。

示例：

```powershell
git tag v0.1.0
git push origin v0.1.0
```

也可以在 GitHub Actions 页面手动运行 `Release` workflow，并填写要发布的 tag。更多说明见 [RELEASE.md](./RELEASE.md)。

## 开源协议

本项目使用 [MIT License](./LICENSE)。

## 说明

Codex Token Monitor 是社区工具，不是 OpenAI 官方产品。项目只分析本机日志中的用量字段，实际计费与额度请以官方账户和平台数据为准。
