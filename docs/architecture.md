[English](architecture_en.md) | [中文](architecture.md)

# 🏗️ 核心架构与流量路由说明 (Architecture)

Cascade Engine 是一个高度复杂的网络层应用，其核心优势在于**智能分流**与**多协议自适应接管**。本篇将深入解析其底层架构。

## 🗺️ 流量转发路径图

```mermaid
graph TD
    A[系统流量 (Browser, Apps)] -->|接管| B(Cascade Engine :10808)
    B --> C{Mixed Port 解析器}
    C -->|HTTP / CONNECT| D[提取 Header Host]
    C -->|SOCKS5| E[提取 ATYP 目标地址]
    D --> F{Routing Engine}
    E --> F
    F -->|匹配国内域名 (.cn, baidu...)| G[Direct 直连模式]
    G --> H((目标网站))
    F -->|未匹配 (海外流量)| I[双重级联模式]
    I --> J[连接本地 VPN 代理]
    J --> K[发送 Socks5 穿透指令]
    K --> L[连接海外 ISP 节点]
    L --> M[Method 0x02 账密鉴权]
    M --> H
```

## 🔀 Mixed Port 自适应协议解析

为了实现像 Clash 一样强悍的接管能力，Cascade 抛弃了单一的 SOCKS5 监听，升级为**混合端口 (Mixed Port)** 架构：

1. **协议嗅探 (Protocol Peeking)**：
   客户端连接建立后，Rust 内核会利用 `TcpStream::peek` 读取第一个字节。
   - 若首字节为 `0x05`，进入 SOCKS5 严格握手状态机。
   - 若首字节为 ASCII 大写字母（如 `C` 代表 `CONNECT`，`G` 代表 `GET`），进入 HTTP 代理握手阶段。
2. **连接归一化**：
   无论上游是哪种协议，解析器最终都会将目标地址转换为统一的格式，并向原客户端返回对应的成功响应（`0x05 0x00...` 或是 `HTTP/1.1 200 Connection Established`），随后将干净的流量丢给 Routing Engine 进行透传。

## 🧠 Routing Engine 智能分流大脑

在获取到客户端真实意图访问的 Host (域名或 IP) 后，流量会进入智能分流判定：

- **直连规则 (Direct)**：引擎内置了 MVP 匹配规则（包含 `.cn`, `baidu.com`, `localhost` 等）。如果目标符合规则，引擎会放弃建立繁琐的代理隧道，直接使用 Tokio 的 `TcpStream::connect` 与目标握手，享受零开销的物理宽带极限速度。
- **级联隧道 (Cascade Tunnel)**：对于海外流量，引擎会先连接本地的 VPN 端口，随后在同一个 TCP 流中发送指令打通海外 ISP，并完成 Method 0x02 鉴权，最后再将真实的流量投递进去。

## 💻 系统分层接管策略

Cascade 提供两种维度的流量接管，互不干扰：

1. **WinINet 全局接管**：
   通过写入 Windows 注册表 `Internet Settings`，使用标准 HTTP 代理格式 (`127.0.0.1:10808`)，并调用 Win32 API 强制系统网络缓存刷新，接管所有常规浏览器的流量。
2. **CLI 精确注入 (Hook)**：
   由于全局环境变量 (`HTTP_PROXY`) 会导致很多不可预知的环境污染，Cascade 提供了针对 `Git` 和 `NPM` 的独立代理开关。通过调用底层命令 (`git config`, `npm config`)，精准配置终端开发工具的网络环境。