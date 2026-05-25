[English](tech_stack_en.md) | [中文](tech_stack.md)

# 🛠️ Tech Stack & Choices

Cascade Engine is committed to creating a geek-level, high-performance, and low-resource-footprint desktop network proxy tool. To achieve this, we have adopted the most cutting-edge modern tech stacks for both the frontend and backend.

## 🎨 Frontend

| Technology / Framework | Why we chose this |
| :--- | :--- |
| **Tauri v2** | Discarding the bloated Electron (Chromium + Node.js), Tauri uses the system's native Webview to render the frontend. This keeps Cascade's installer tiny and memory usage exceptionally low. |
| **React + TypeScript** | The industry's most mature UI framework. Combined with a strong type system (TS), it ensures absolute stability during complex component lifecycles and state transitions (e.g., config sync, start/stop management). |
| **Vite** | A lightning-fast modern frontend build tool. It provides instant cold starts and Hot Module Replacement (HMR), saying goodbye to the long waits of Webpack. |
| **TailwindCSS v4** | A utility-first CSS framework. It allows us to build geeky, modern, and responsive dark-themed UI panels in record time without maintaining massive, messy CSS files. |

## ⚙️ Backend

| Technology / Library | Why we chose this |
| :--- | :--- |
| **Rust** | A memory-safe, GC-free system-level programming language. When handling highly concurrent network packets, Rust provides C/C++ level performance with minimal latency, perfectly suited for the core network routing engine. |
| **Tokio** | The most powerful asynchronous runtime in the Rust ecosystem. Leveraging `tokio::net::TcpStream` and `tokio::io::copy_bidirectional`, it achieves zero-cost bidirectional traffic relay and non-blocking I/O. |
| **winreg** | Used for manipulating the Windows Registry directly. It writes `ProxyEnable` and `ProxyServer` dynamically without external scripts, achieving seamless global traffic takeover. |
| **windows-sys** | Direct invocation of underlying Win32 APIs (`InternetSetOptionW`). Simply modifying the registry is not enough; this library sends broadcast notifications forcing the system and all apps to instantly refresh their network cache. |
| **tauri-plugin-store** | The official persistent storage plugin for Tauri. It securely saves user configurations such as VPN ports, remote ISP nodes, and auth credentials to the local disk. |