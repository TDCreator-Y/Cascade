import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [status, setStatus] = useState<"idle" | "running" | "success" | "error">("idle");
  const [message, setMessage] = useState<string>("");

  const handleStartCascade = async () => {
    try {
      setStatus("running");
      // 调用 Rust 端定义的 start_cascade command
      const res = await invoke<string>("start_cascade");
      setStatus("success");
      setMessage(res);
    } catch (err) {
      setStatus("error");
      setMessage(String(err));
    }
  };

  return (
    <main className="min-h-screen flex flex-col items-center justify-center bg-zinc-950 text-zinc-50 p-6">
      <div className="max-w-md w-full flex flex-col items-center space-y-8">
        {/* Logo / Header */}
        <div className="flex flex-col items-center space-y-4">
          <div className="w-16 h-16 rounded-2xl bg-gradient-to-tr from-cyan-500 to-blue-600 shadow-lg shadow-cyan-500/20 flex items-center justify-center">
            <svg
              className="w-8 h-8 text-white"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 10V3L4 14h7v7l9-11h-7z"
              />
            </svg>
          </div>
          <h1 className="text-3xl font-bold tracking-tight text-white">Cascade</h1>
          <p className="text-zinc-400 text-sm text-center">
            Advanced Network Proxy Engine
          </p>
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
                  : "bg-rose-500/10 border-rose-500/20 text-rose-400"
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