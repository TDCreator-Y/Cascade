import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LazyStore } from "@tauri-apps/plugin-store";
import "./App.css";

const store = new LazyStore("config.json");

function App() {
  const [status, setStatus] = useState<"idle" | "running" | "success" | "error">("idle");
  const [message, setMessage] = useState<string>("");

  const [vpnPort, setVpnPort] = useState(7897);
  const [ispIp, setIspIp] = useState("64.51.26.139");
  const [ispPort, setIspPort] = useState(443);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const savedVpnPort = await store.get<number>("vpnPort");
        const savedIspIp = await store.get<string>("ispIp");
        const savedIspPort = await store.get<number>("ispPort");
        const savedUsername = await store.get<string>("username");
        const savedPassword = await store.get<string>("password");

        if (savedVpnPort) setVpnPort(savedVpnPort);
        if (savedIspIp) setIspIp(savedIspIp);
        if (savedIspPort) setIspPort(savedIspPort);
        if (savedUsername) setUsername(savedUsername);
        if (savedPassword) setPassword(savedPassword);
      } catch (e) {
        console.error("Failed to load config", e);
      }
    };
    loadConfig();
  }, []);

  const handleSaveConfig = async () => {
    try {
      await store.set("vpnPort", vpnPort);
      await store.set("ispIp", ispIp);
      await store.set("ispPort", ispPort);
      await store.set("username", username);
      await store.set("password", password);
      await store.save();
      setMessage("配置已保存到本地");
      setTimeout(() => setMessage(""), 3000);
    } catch (e) {
      setMessage("保存失败: " + String(e));
    }
  };

  const handleStartCascade = async () => {
    try {
      setStatus("running");
      // 传递动态参数给 Rust 端
      const res = await invoke<string>("start_cascade", {
        vpnPort,
        ispIp,
        ispPort,
        username,
        password,
      });
      setStatus("success");
      setMessage(res);
    } catch (err) {
      setStatus("error");
      setMessage(String(err));
    }
  };

  return (
    <main className="min-h-screen flex flex-col items-center justify-center bg-zinc-950 text-zinc-50 p-6">
      <div className="max-w-md w-full flex flex-col items-center space-y-6">
        {/* Header & Status Light */}
        <div className="flex flex-col items-center space-y-4 w-full">
          <div className="flex items-center justify-between w-full">
            <h1 className="text-2xl font-bold tracking-tight text-white flex items-center space-x-3">
              <span>Cascade Config</span>
              <span className="relative flex h-3 w-3">
                {(status === "running" || status === "success") && (
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                )}
                <span
                  className={`relative inline-flex rounded-full h-3 w-3 ${
                    status === "running" || status === "success"
                      ? "bg-emerald-500"
                      : status === "error"
                      ? "bg-rose-500"
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
        </div>

        {/* Configuration Form */}
        <div className="w-full space-y-4 bg-zinc-900/50 p-6 rounded-2xl border border-zinc-800/50">
          <div className="space-y-1">
            <label className="text-xs text-zinc-400 font-medium">本地 VPN 端口</label>
            <input
              type="number"
              value={vpnPort}
              onChange={(e) => setVpnPort(Number(e.target.value))}
              className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
            />
          </div>
          
          <div className="grid grid-cols-3 gap-4">
            <div className="col-span-2 space-y-1">
              <label className="text-xs text-zinc-400 font-medium">远程 ISP IP</label>
              <input
                type="text"
                value={ispIp}
                onChange={(e) => setIspIp(e.target.value)}
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
              />
            </div>
            <div className="col-span-1 space-y-1">
              <label className="text-xs text-zinc-400 font-medium">端口</label>
              <input
                type="number"
                value={ispPort}
                onChange={(e) => setIspPort(Number(e.target.value))}
                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
              />
            </div>
          </div>

          <div className="space-y-1">
            <label className="text-xs text-zinc-400 font-medium">ISP 用户名</label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
            />
          </div>

          <div className="space-y-1">
            <label className="text-xs text-zinc-400 font-medium">ISP 密码</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors"
            />
          </div>
        </div>

        {/* Action Button */}
        <button
          onClick={handleStartCascade}
          disabled={status === "running"}
          className={`
            w-full py-3 px-6 rounded-xl font-medium text-sm transition-all duration-300
            flex items-center justify-center space-x-2
            ${
              status === "running"
                ? "bg-zinc-800 text-zinc-400 cursor-not-allowed"
                : "bg-white text-zinc-950 hover:bg-zinc-200 hover:scale-[1.02] active:scale-[0.98] shadow-xl shadow-white/10"
            }
          `}
        >
          {status === "running" ? (
            <>
              <svg className="animate-spin h-4 w-4 text-zinc-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <span>Starting Engine...</span>
            </>
          ) : (
            <span>启动 Cascade</span>
          )}
        </button>

        {/* Status Message */}
        {message && (
          <div
            className={`
              w-full p-4 rounded-xl text-sm border
              ${
                status === "success"
                  ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-400"
                  : status === "error"
                  ? "bg-rose-500/10 border-rose-500/20 text-rose-400"
                  : "bg-blue-500/10 border-blue-500/20 text-blue-400"
              }
            `}
          >
            {message}
          </div>
        )}
      </div>
    </main>
  );
}

export default App;