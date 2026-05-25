[English](user_guide_en.md) | [中文](user_guide.md)

# 📖 Cascade User Guide

Welcome to Cascade Engine! This guide will take you from scratch to configuring and using this powerful double-cascade network proxy tool.

## ⚙️ Step 1: Prerequisites

Cascade is a **double-cascade proxy** tool, which means its overseas network penetration is built upon your "first-hop" node.
Before using Cascade, ensure that:
1. You are already running a basic VPN / Proxy software locally (e.g., Clash, V2Ray).
2. You know the local SOCKS5 proxy port exposed by this software (commonly `7890` or `7897`).

## 🚀 Step 2: Basic Usage & Configuration

Open the Cascade client, and you will see a minimalist dark configuration panel. Please fill it out according to the following steps:

1. **Local VPN Port**: Enter the local port of your first-hop proxy (e.g., `7897`).
2. **Remote ISP IP & Port**: Enter the IP and port of the final overseas pure ISP node you want to cascade to (e.g., `64.51.26.139` and `443`).
3. **ISP Credentials**: Enter the Username and Password assigned to you for the remote ISP node.
4. **Save Config**: Click `Save Config` in the top right corner. The information will be securely persisted to the local `config.json` so you don't have to re-enter it next time.
5. **One-Click Start**: Click the white `Start Cascade` button.
   - The status light turns **Green**.
   - Cascade instantly modifies the Windows Registry and refreshes the network cache.
   - **Congratulations! Your system's global network is now perfectly taken over by Cascade.**

## 🛠️ Step 3: Advanced Usage (Independent CLI Takeover)

As a developer, global environment variables (like setting the `HTTP_PROXY` system variable) often lead to disastrous consequences (causing exceptions in local scripts, Docker image pulls, and intranet testing services).

Cascade provides a much more elegant solution: **Precise CLI Hook**.

At the bottom of the main panel, you will see two toggles:
- **Git Proxy**
- **NPM Proxy**

### How to use it?
- Simply click the toggles; no need to restart your terminal.
- When enabled, Cascade automatically executes `git config` and `npm config` commands under the hood, allowing `git clone` and `npm install` to enjoy the pure overseas network.
- When you stop the Cascade engine, **all configurations are automatically reverted**, leaving absolutely no garbage configurations to pollute your dev environment!

---

*If you encounter a red status light or an error during startup, please check if your first-hop local VPN is running normally and if the remote ISP credentials are correct.*