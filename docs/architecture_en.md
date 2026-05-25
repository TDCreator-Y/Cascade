[English](architecture_en.md) | [中文](architecture.md)

# 🏗️ Core Architecture & Traffic Routing

Cascade Engine is a highly sophisticated network-layer application. Its core strengths lie in **Smart Routing** and **Multi-Protocol Adaptive Takeover**. This document dives deep into its underlying architecture.

## 🗺️ Traffic Forwarding Flowchart

```mermaid
graph TD
    A[System Traffic (Browser, Apps)] -->|Takeover| B(Cascade Engine :10808)
    B --> C{Mixed Port Parser}
    C -->|HTTP / CONNECT| D[Extract Header Host]
    C -->|SOCKS5| E[Extract ATYP Address]
    D --> F{Routing Engine}
    E --> F
    F -->|Match Domestic Domains (.cn, baidu...)| G[Direct Mode]
    G --> H((Target Website))
    F -->|Unmatched (Overseas Traffic)| I[Double Cascade Mode]
    I --> J[Connect Local VPN Proxy]
    J --> K[Send Socks5 Penetration Command]
    K --> L[Connect Overseas ISP Node]
    L --> M[Method 0x02 Auth]
    M --> H
```

## 🔀 Mixed Port Adaptive Protocol Parsing

To achieve commercial-grade takeover capabilities like Clash, Cascade abandoned a single SOCKS5 listener in favor of a **Mixed Port** architecture:

1. **Protocol Peeking**:
   Upon a client connection, the Rust core uses `TcpStream::peek` to read the first byte.
   - If the first byte is `0x05`, it enters the strict SOCKS5 handshake state machine.
   - If the first byte is an uppercase ASCII letter (e.g., `C` for `CONNECT`, `G` for `GET`), it enters the HTTP proxy handshake phase.
2. **Connection Normalization**:
   Regardless of the upstream protocol, the parser standardizes the target address format and returns the corresponding success response to the client (`0x05 0x00...` or `HTTP/1.1 200 Connection Established`). It then passes the raw, clean traffic to the Routing Engine.

## 🧠 Routing Engine

After extracting the real Host (domain or IP) the client intends to visit, the traffic enters the smart routing decision phase:

- **Direct Rule**: The engine has built-in MVP matching rules (including `.cn`, `baidu.com`, `localhost`, etc.). If the target matches, the engine skips the complex proxy tunnel creation and directly uses Tokio's `TcpStream::connect` to handshake with the target, enjoying zero-overhead maximum bandwidth speed.
- **Cascade Tunnel**: For overseas traffic, the engine first connects to the local VPN port. Then, within the exact same TCP stream, it sends commands to pierce through the overseas ISP, completes Method 0x02 authentication, and finally delivers the real payload.

## 💻 Layered System Takeover Strategy

Cascade offers two dimensions of traffic takeover without interfering with each other:

1. **WinINet Global Takeover**:
   By writing to the Windows Registry `Internet Settings` using standard HTTP proxy format (`127.0.0.1:10808`) and calling Win32 APIs to force a system network cache refresh, it takes over all conventional browser traffic seamlessly.
2. **Precise CLI Hook**:
   Since global environment variables (`HTTP_PROXY`) can cause unpredictable environment pollution, Cascade provides independent proxy toggles for `Git` and `NPM`. By invoking underlying commands (`git config`, `npm config`), it precisely configures the network environment for terminal development tools.