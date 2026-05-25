[English](tech_stack_en.md) | [中文](tech_stack.md)

# 🛠️ 技术栈与选型说明 (Tech Stack)

Cascade Engine 致力于打造极客级别、高性能、低资源占用的桌面端网络代理工具。为此，我们在前后端选型上均采用了当前业界最前沿的现代化技术栈。

## 🎨 前端选型 (Frontend)

| 技术 / 框架 | 选型原因 (Why we chose this) |
| :--- | :--- |
| **Tauri v2** | 摒弃了臃肿的 Electron (Chromium + Node.js)，Tauri 采用系统原生的 Webview 渲染前端。这让 Cascade 的安装包极小，内存占用极低。 |
| **React + TypeScript** | 业界最成熟的 UI 构建框架。配合强类型系统 (TS)，在复杂的组件生命周期与状态流转（如配置中心联动、启停状态管理）中保证了绝对的稳定。 |
| **Vite** | 极速的现代前端构建工具。提供闪电般的冷启动和热重载 (HMR) 体验，告别 Webpack 的漫长等待。 |
| **TailwindCSS v4** | 原子化 CSS 框架。让我们能在极短的时间内编写出极客风、现代化、响应式的暗色 UI 面板，而无需维护庞大且混乱的 CSS 文件。 |

## ⚙️ 后端选型 (Backend)

| 技术 / 库 | 选型原因 (Why we chose this) |
| :--- | :--- |
| **Rust** | 内存安全、无垃圾回收 (GC) 的系统级编程语言。在处理高并发网络数据包时，Rust 提供了媲美 C/C++ 的性能和极低的延迟，完美胜任底层网络路由引擎的核心。 |
| **Tokio** | Rust 生态中最强大的异步运行时。借助 `tokio::net::TcpStream` 和 `tokio::io::copy_bidirectional`，实现了零开销的双向流量透传和非阻塞 I/O。 |
| **winreg** | 用于在 Windows 环境下直接操纵系统注册表，无需依赖外部脚本即可动态写入 `ProxyEnable` 和 `ProxyServer`，实现全局流量的无缝接管。 |
| **windows-sys** | 直接调用底层 Win32 API (`InternetSetOptionW`)。仅仅修改注册表是不够的，必须通过该库主动下发广播通知，强制系统和所有应用程序立即刷新网络缓存，使代理配置秒级生效。 |
| **tauri-plugin-store** | Tauri 官方的持久化存储插件。安全可靠地将用户的 VPN 端口、远程 ISP 节点、鉴权账密等信息落盘保存。 |