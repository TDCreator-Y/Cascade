import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { LazyStore } from "@tauri-apps/plugin-store";
import "./App.css";

const store = new LazyStore("config.json");

const IPV4_REGEX = /^(\d{1,3}\.){3}\d{1,3}$/;

function isValidIpv4(ip: string): boolean {
  if (!IPV4_REGEX.test(ip)) return false;
  return ip.split(".").every((n) => Number(n) <= 255);
}

function isValidPort(port: number): boolean {
  return Number.isInteger(port) && port >= 1 && port <= 65535;
}

function App() {
  const [status, setStatus] = useState<"idle" | "running" | "stopped" | "error">("idle");
  const [message, setMessage] = useState<string>("");

  const [vpnPort, setVpnPort] = useState(7897);
  const [ispIp, setIspIp] = useState("64.51.26.139");
  const [ispPort, setIspPort] = useState(443);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const [claudeProxyEnabled, setClaudeProxyEnabled] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [validationError, setValidationError] = useState<string>("");

  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const savedVpnPort = await store.get<number>("vpnPort");
        const savedIspIp = await store.get<string>("ispIp");
        const savedIspPort = await store.get<number>("ispPort");
        const savedUsername = await store.get<string>("username");
        const savedPassword = await store.get<string>("password");
        const savedClaudeProxy = await store.get<boolean>("claudeProxyEnabled");

        if (savedVpnPort) setVpnPort(savedVpnPort);
        if (savedIspIp) setIspIp(savedIspIp);
        if (savedIspPort) setIspPort(savedIspPort);
        if (savedUsername) setUsername(savedUsername);
        if (savedPassword) setPassword(savedPassword);
        if (savedClaudeProxy !== undefined) setClaudeProxyEnabled(savedClaudeProxy);
      } catch (e) {
        console.error("Failed to load config", e);
      }
    };
    loadConfig();
  }, []);

  // 监听来自 Rust 的实时日志事件
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<string>("cascade-log", (event) => {
      setLogs((prev) => {
        const next = [event.payload, ...prev];
        return next.slice(0, 200);
      });
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // 日志新增时自动滚动到顶部（最新在上）
  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = 0;
    }
  }, [logs]);

  const handleSaveConfig = async () => {
    try {
      await store.set("vpnPort", vpnPort);
      await store.set("ispIp", ispIp);
      await store.set("ispPort", ispPort);
      await store.set("username", username);
      await store.set("password", password);
      await store.set("claudeProxyEnabled", claudeProxyEnabled);
      await store.save();
      setMessage("配置已保存");
      setTimeout(() => setMessage(""), 2000);
    } catch (e) {
      setMessage("保存失败: " + String(e));
    }
  };

  const validateForm = (): boolean => {
    if (!isValidPort(vpnPort)) {
      setValidationError("本地 VPN 端口无效，请输入 1–65535 之间的整数");
      return false;
    }
    if (!isValidIpv4(ispIp)) {
      setValidationError("远程 ISP IP 格式不正确，请输入合法的 IPv4 地址（如 64.51.26.139）");
      return false;
    }
    if (!isValidPort(ispPort)) {
      setValidationError("远程 ISP 端口无效，请输入 1–65535 之间的整数");
      return false;
    }
    if (!username.trim()) {
      setValidationError("ISP 用户名不能为空");
      return false;
    }
    if (!password) {
      setValidationError("ISP 密码不能为空");
      return false;
    }
    setValidationError("");
    return true;
  };

  const handleStartCascade = async () => {
    if (!validateForm()) return;
    try {
      setStatus("running");
      setMessage("");
      setLogs([]);
      const res = await invoke<string>("start_cascade", {
        vpnPort,
        ispIp,
        ispPort,
        username,
        password,
        claudeProxyEnabled,
      });
      setStatus("running");
      setMessage(res);
    } catch (err) {
      setStatus("error");
      setMessage(String(err));
    }
  };

  const handleStopCascade = async () => {
    try {
      const res = await invoke<string>("stop_cascade");
      setStatus("stopped");
      setMessage(res);
    } catch (err) {
      setStatus("error");
      setMessage("停止失败: " + String(err));
    }
  };

  const handleToggleClaude = async (enable: boolean) => {
    setClaudeProxyEnabled(enable);
    if (status === "running") {
      try {
        await invoke("toggle_claude_proxy", { enable });
      } catch (err) {
        console.error(err);
      }
    }
  };

  const isRunning = status === "running";

  return (
    <main className="min-h-screen flex flex-col items-center bg-zinc-950 text-zinc-50 p-6 pt-8">
      <div className="max-w-md w-full flex flex-col space-y-5">
        {/* Header & Status */}
        <div className="flex items-center justify-between w-full">
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center space-x-3">
            <span>Cascade Config</span>
            <span className="relative flex h-3 w-3">
              {isRunning && (
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
              )}
              <span
                className={`relative inline-flex rounded-full h-3 w-3 ${
                  isRunning
                    ? "bg-emerald-500"
                    : status === "error"
                    ? "bg-rose-500"
                    : status === "stopped"
                    ? "bg-amber-500"
                    : "bg-zinc-600"
                }`}
              ></span>
            </span>
          </h1>
          <button
            onClick={handleSaveConfig}
            className="text-xs font-medium px-3 py-1.5 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
          >
            保存配置
          </button>
        </div>

        {/* Configuration Form */}
        <div className="w-full space-y-4 bg-zinc-900/50 p-6 rounded-2xl border border-zinc-800/50">
          <div className="space-y-1">
            <label htmlFor="vpn-port" className="text-xs text-zinc-400 font-medium">本地 VPN 端口</label>
            <input
              id="vpn-port"
              type="number"
              value={vpnPort}
              min={1}
              max={65535}
              onChange={(e) => { setVpnPort(Number(e.target.value)); setValidationError(""); }}
              className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
            />
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div className="col-span-2 space-y-1">
              <label htmlFor="isp-ip" className="text-xs text-zinc-400 font-medium">远程 ISP IP</label>
              <input
                id="isp-ip"
                type="text"
                value={ispIp}
                placeholder="0.0.0.0"
                onChange={(e) => { setIspIp(e.target.value); setValidationError(""); }}
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
              />
            </div>
            <div className="col-span-1 space-y-1">
              <label htmlFor="isp-port" className="text-xs text-zinc-400 font-medium">端口</label>
              <input
                id="isp-port"
                type="number"
                value={ispPort}
                min={1}
                max={65535}
                onChange={(e) => { setIspPort(Number(e.target.value)); setValidationError(""); }}
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
              />
            </div>
          </div>

          <div className="space-y-1">
            <label htmlFor="username" className="text-xs text-zinc-400 font-medium">ISP 用户名</label>
            <input
              id="username"
              type="text"
              value={username}
              onChange={(e) => { setUsername(e.target.value); setValidationError(""); }}
              className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
            />
          </div>

          <div className="space-y-1">
            <label htmlFor="password" className="text-xs text-zinc-400 font-medium">ISP 密码</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => { setPassword(e.target.value); setValidationError(""); }}
              className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
            />
            <p className="text-xs text-amber-500/70 mt-1">
              ⚠ 密码以明文保存在本地 config.json，请勿在共享设备上使用
            </p>
          </div>
        </div>

        {/* Validation Error */}
        {validationError && (
          <div className="w-full p-3 rounded-xl text-xs border bg-rose-500/10 border-rose-500/20 text-rose-400">
            {validationError}
          </div>
        )}

        {/* Claude CLI Proxy Toggle */}
        <div className="w-full">
          <div
            onClick={() => handleToggleClaude(!claudeProxyEnabled)}
            className={`cursor-pointer p-4 rounded-xl border transition-all duration-300 flex items-center justify-between ${
              claudeProxyEnabled
                ? "bg-cyan-500/10 border-cyan-500/30"
                : "bg-zinc-900/50 border-zinc-800/50"
            }`}
          >
            <div className="flex items-center space-x-3">
              <div
                className={`w-8 h-8 rounded-lg flex items-center justify-center ${
                  claudeProxyEnabled ? "bg-cyan-500/20 text-cyan-400" : "bg-zinc-800 text-zinc-400"
                }`}
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="2"
                    d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                  />
                </svg>
              </div>
              <div className="flex flex-col">
                <span className="text-sm font-medium">Claude (CLI) 代理</span>
                <span className="text-xs text-zinc-500">为终端注入环境变量，适配 Claude Code</span>
              </div>
            </div>
            <div
              className={`w-10 h-6 rounded-full transition-colors relative ${
                claudeProxyEnabled ? "bg-cyan-500" : "bg-zinc-700"
              }`}
            >
              <div
                className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-transform ${
                  claudeProxyEnabled ? "left-5" : "left-1"
                }`}
              ></div>
            </div>
          </div>
        </div>

        {/* Action Button */}
        {isRunning ? (
          <button
            onClick={handleStopCascade}
            className="w-full py-3 px-6 rounded-xl font-medium text-sm transition-all duration-300 flex items-center justify-center space-x-2 bg-rose-500/10 text-rose-500 hover:bg-rose-500/20 hover:scale-[1.02] active:scale-[0.98]"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <rect x="6" y="6" width="12" height="12" rx="2" strokeWidth="2" />
            </svg>
            <span>停止 Cascade</span>
          </button>
        ) : (
          <button
            onClick={handleStartCascade}
            className="w-full py-3 px-6 rounded-xl font-medium text-sm transition-all duration-300 flex items-center justify-center space-x-2 bg-white text-zinc-950 hover:bg-zinc-200 hover:scale-[1.02] active:scale-[0.98] shadow-xl shadow-white/10"
          >
            <span>启动 Cascade</span>
          </button>
        )}

        {/* Status Message */}
        {message && (
          <div
            className={`w-full p-4 rounded-xl text-sm border ${
              isRunning
                ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-400"
                : status === "error"
                ? "bg-rose-500/10 border-rose-500/20 text-rose-400"
                : status === "stopped"
                ? "bg-amber-500/10 border-amber-500/20 text-amber-400"
                : "bg-blue-500/10 border-blue-500/20 text-blue-400"
            }`}
          >
            {message}
          </div>
        )}

        {/* Connection Log Panel */}
        {logs.length > 0 && (
          <div className="w-full space-y-1">
            <div className="flex items-center justify-between">
              <span className="text-xs text-zinc-500 font-medium">连接日志</span>
              <button
                onClick={() => setLogs([])}
                className="text-xs text-zinc-600 hover:text-zinc-400 transition-colors"
              >
                清空
              </button>
            </div>
            <div
              ref={logRef}
              className="bg-zinc-900/50 border border-zinc-800/50 rounded-xl p-3 h-48 overflow-y-auto font-mono"
            >
              {logs.map((log, i) => (
                <div
                  key={i}
                  className={`text-xs leading-5 ${
                    log.startsWith("[错误]")
                      ? "text-rose-400"
                      : log.startsWith("[级联]")
                      ? "text-cyan-400"
                      : log.startsWith("[直连]")
                      ? "text-emerald-400"
                      : "text-zinc-400"
                  }`}
                >
                  {log}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </main>
  );
}

export default App;
