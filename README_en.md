[English](README_en.md) | [中文](README.md)

<div align="center">
  <img src="https://via.placeholder.com/150?text=Cascade+Logo" alt="Cascade Logo" width="150"/>
  <h1>Cascade Engine v1.0</h1>
  <p><em>A modern, smart cascade network proxy engine built with Tauri + Rust, designed exclusively for pure overseas IP environments.</em></p>
</div>

---

## 🚀 Core Features

- 🔗 **Double Cascade Tunnel**: Break through network blocks by establishing a double Socks5 tunnel via a local VPN and a remote pure ISP node.
- 🧠 **Smart Routing**: Millisecond-level Host extraction. Direct connection for domestic domains, cascade tunneling for overseas domains.
- 🔀 **HTTP/Socks5 Mixed Port**: A single port (`10808`) perfectly adapts to HTTP CONNECT, plain HTTP, and SOCKS5 proxy requests, rivaling commercial-grade cores.
- 💻 **System Proxy Takeover**: Dynamically modifies Windows Registry and WinINet cache to seamlessly take over global traffic upon launch.
- 🛠️ **Precise CLI Hook**: One-click UI toggles for independent Git / NPM proxy configuration, saying NO to nasty global environment variable pollution.

---

## ⚡ Quick Start

### UI Preview
<div align="center">
  <img src="https://via.placeholder.com/600x400?text=Cascade+UI+Preview" alt="UI Preview" width="600"/>
</div>

### Installation & Build

Ensure you have [Node.js](https://nodejs.org/) and the [Rust Toolchain](https://rustup.rs/) installed.

```bash
# 1. Clone the repository
git clone https://github.com/TDCreator-Y/Cascade.git
cd Cascade

# 2. Install frontend dependencies
npm install

# 3. Start development server
npm run tauri dev

# 4. Build for production
npm run tauri build
```

---

## 📚 Documentation

Want to dive deeper into the underlying principles and advanced usage of Cascade? Check out the documents below:

- 🛠️ [Tech Stack & Choices](docs/tech_stack_en.md)
- 🏗️ [Core Architecture & Traffic Routing](docs/architecture_en.md)
- 📖 [User Guide](docs/user_guide_en.md)

---
<div align="center">
  Made with ❤️ by TDCreator-Y
</div>