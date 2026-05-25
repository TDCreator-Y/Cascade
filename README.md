[English](README_en.md) | [中文](README.md)

<div align="center">
  <img src="https://via.placeholder.com/150?text=Cascade+Logo" alt="Cascade Logo" width="150"/>
  <h1>Cascade Engine v1.0</h1>
  <p><em>基于 Tauri + Rust 的现代化智能级联网络代理引擎，专为纯净海外 IP 环境打造。</em></p>
</div>

---

## 🚀 核心特性 (Core Features)

- 🔗 **双重级联隧道 (Double Cascade Tunnel)**: 突破网络封锁，通过本地 VPN 与远程纯净 ISP 节点建立双重 Socks5 隧道。
- 🧠 **智能分流大脑 (Smart Routing)**: 毫秒级提取请求 Host，国内域名直连，海外域名走级联隧道。
- 🔀 **混合端口支持 (HTTP/Socks5 Mixed Port)**: 单一端口 (`10808`) 完美自适应 HTTP CONNECT、普通 HTTP 及 SOCKS5 代理请求，媲美商业级内核。
- 💻 **无感接管系统代理 (System Proxy Takeover)**: 动态修改 Windows 注册表与 WinINet 缓存，启动即接管全局流量。
- 🛠️ **CLI 工具精确注入 (Precise CLI Hook)**: 提供界面一键开关，独立配置 Git / NPM 代理，拒绝恶心的全局环境变量污染。

---

## ⚡ 快速开始 (Quick Start)

### 界面预览
<div align="center">
  <img src="https://via.placeholder.com/600x400?text=Cascade+UI+Preview" alt="UI Preview" width="600"/>
</div>

### 安装与编译

确保你已经安装了 [Node.js](https://nodejs.org/) 和 [Rust 工具链](https://rustup.rs/)。

```bash
# 1. 克隆仓库
git clone https://github.com/TDCreator-Y/Cascade.git
cd Cascade

# 2. 安装前端依赖
npm install

# 3. 启动开发环境
npm run tauri dev

# 4. 构建生产版本
npm run tauri build
```

---

## 📚 文档目录 (Documentation)

想要深入了解 Cascade 的底层原理和高级玩法？请查阅以下文档：

- 🛠️ [技术栈与选型说明 (Tech Stack)](docs/tech_stack.md)
- 🏗️ [核心架构与流量路由说明 (Architecture)](docs/architecture.md)
- 📖 [用户使用指南 (User Guide)](docs/user_guide.md)

---
<div align="center">
  Made with ❤️ by TDCreator-Y
</div>