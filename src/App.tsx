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

  const [gitProxy, setGitProxy] = useState(false);
  const [npmProxy, setNpmProxy] = useState(false);

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const savedVpnPort = await store.get<number>("vpnPort");
        const savedIspIp = await store.get<string>("ispIp");
        const savedIspPort = await store.get<number>("ispPort");
        const savedUsername = await store.get<string>("username");
        const savedPassword = await store.get<string>("password");
        const savedGitProxy = await store.get<boolean>("gitProxy");
        const savedNpmProxy = await store.get<boolean>("npmProxy");

        if (savedVpnPort) setVpnPort(savedVpnPort);
        if (savedIspIp) setIspIp(savedIspIp);
        if (savedIspPort) setIspPort(savedIspPort);
        if (savedUsername) setUsername(savedUsername);
        if (savedPassword) setPassword(savedPassword);
        if (savedGitProxy !== undefined) setGitProxy(savedGitProxy);
        if (savedNpmProxy !== undefined) setNpmProxy(savedNpmProxy);
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
      await store.set("gitProxy", gitProxy);
      await store.set("npmProxy", npmProxy);
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
        gitProxy,
        npmProxy,
      });
      setStatus("success");
      setMessage(res);
    } catch (err) {
      setStatus("error");
      setMessage(String(err));
    }
  };

  const handleStopCascade = async () => {
    try {
      await invoke("stop_cascade");
      setStatus("idle");
      setMessage("Cascade Engine 已停止，系统代理及 CLI 代理已恢复");
      setTimeout(() => setMessage(""), 3000);
    } catch (err) {
      setMessage("停止失败: " + String(err));
    }
  };

  const handleToggleGit = async (enable: boolean) => {
    setGitProxy(enable);
    if (status === "running" || status === "success") {
      try {
        await invoke("toggle_git_proxy", { enable });
      } catch (err) {
        console.error(err);
      }
    }
  };

  const handleToggleNpm = async (enable: boolean) => {
    setNpmProxy(enable);
    if (status === "running" || status === "success") {
      try {
        await invoke("toggle_npm_proxy", { enable });
      } catch (err) {
        console.error(err);
      }
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

        {/* CLI Proxy Toggles */}
        <div className="w-full grid grid-cols-2 gap-4">
          <div 
            onClick={() => handleToggleGit(!gitProxy)}
            className={`cursor-pointer p-4 rounded-xl border transition-all duration-300 flex items-center justify-between ${
              gitProxy ? "bg-cyan-500/10 border-cyan-500/30" : "bg-zinc-900/50 border-zinc-800/50"
            }`}
          >
            <div className="flex items-center space-x-3">
              <div className={`w-8 h-8 rounded-lg flex items-center justify-center ${gitProxy ? "bg-cyan-500/20 text-cyan-400" : "bg-zinc-800 text-zinc-400"}`}>
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                  <path fillRule="evenodd" clipRule="evenodd" d="M11.96 1.056c-6.046 0-10.932 4.887-10.932 10.933 0 4.83 3.132 8.93 7.478 10.378.546.1.745-.236.745-.526 0-.26-.01-1.12-.014-2.046-3.04.66-3.682-1.282-3.682-1.282-.497-1.262-1.214-1.598-1.214-1.598-.992-.68.075-.666.075-.666 1.096.077 1.673 1.126 1.673 1.126.973 1.666 2.553 1.185 3.176.906.1-.705.38-1.185.69-1.458-2.428-.276-4.978-1.214-4.978-5.397 0-1.19.424-2.164 1.12-2.926-.112-.276-.486-1.385.106-2.886 0 0 .915-.293 2.992 1.116a10.395 10.395 0 012.723-.366c.924.004 1.854.125 2.724.366 2.075-1.41 2.99-1.116 2.99-1.116.594 1.501.22 2.61.108 2.886.698.762 1.12 1.736 1.12 2.926 0 4.193-2.553 5.117-4.99 5.388.39.337.74 1.002.74 2.018 0 1.45-.014 2.62-.014 2.973 0 .293.197.632.75.524 4.344-1.452 7.472-5.55 7.472-10.376C22.892 5.943 18.006 1.056 11.96 1.056z" />
                </svg>
              </div>
              <div className="text-sm font-medium">Git 代理</div>
            </div>
            <div className={`w-10 h-6 rounded-full transition-colors relative ${gitProxy ? "bg-cyan-500" : "bg-zinc-700"}`}>
              <div className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-transform ${gitProxy ? "left-5" : "left-1"}`}></div>
            </div>
          </div>

          <div 
            onClick={() => handleToggleNpm(!npmProxy)}
            className={`cursor-pointer p-4 rounded-xl border transition-all duration-300 flex items-center justify-between ${
              npmProxy ? "bg-rose-500/10 border-rose-500/30" : "bg-zinc-900/50 border-zinc-800/50"
            }`}
          >
            <div className="flex items-center space-x-3">
              <div className={`w-8 h-8 rounded-lg flex items-center justify-center ${npmProxy ? "bg-rose-500/20 text-rose-400" : "bg-zinc-800 text-zinc-400"}`}>
                <svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <path d="M4 4H20V20H12V8H8V20H4V4Z" fill="currentColor"/>
                </svg>
              </div>
              <div className="text-sm font-medium">NPM 代理</div>
            </div>
            <div className={`w-10 h-6 rounded-full transition-colors relative ${npmProxy ? "bg-rose-500" : "bg-zinc-700"}`}>
              <div className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-transform ${npmProxy ? "left-5" : "left-1"}`}></div>
            </div>
          </div>
        </div>

        {/* Action Button */}
        {status === "running" || status === "success" ? (
          <button
            onClick={handleStopCascade}
            className="w-full py-3 px-6 rounded-xl font-medium text-sm transition-all duration-300 flex items-center justify-center space-x-2 bg-rose-500/10 text-rose-500 hover:bg-rose-500/20 hover:scale-[1.02] active:scale-[0.98] shadow-xl shadow-rose-500/10"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
              <rect x="6" y="6" width="12" height="12" rx="2" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
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