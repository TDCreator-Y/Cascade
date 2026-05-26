import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  LazyStore: class {
    async get() { return null; }
    async set() { return undefined; }
    async save() { return undefined; }
  },
}));

function fillValidForm() {
  fireEvent.change(screen.getByDisplayValue(""), { target: { value: "testuser" } });
  // password field is the second input with empty value
  const emptyInputs = screen.getAllByDisplayValue("");
  if (emptyInputs.length > 0) {
    fireEvent.change(emptyInputs[0], { target: { value: "testpass" } });
  }
}

describe("App Component", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the main interface correctly", () => {
    render(<App />);
    expect(screen.getByText("Cascade Config")).toBeDefined();
    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    expect(startButton).toBeDefined();
  });

  it("shows validation error when username is empty", async () => {
    render(<App />);
    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(screen.getByText(/ISP 用户名不能为空/i)).toBeDefined();
    });
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("shows validation error for invalid IP", async () => {
    render(<App />);
    const ipInput = screen.getByDisplayValue("64.51.26.139");
    fireEvent.change(ipInput, { target: { value: "999.999.999.999" } });

    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(screen.getByText(/ISP IP 格式不正确/i)).toBeDefined();
    });
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("handles successful cascade engine start", async () => {
    vi.mocked(invoke).mockResolvedValueOnce("Cascade Engine 已启动，系统代理已接管");

    render(<App />);

    // Fill required fields
    const usernameInput = screen.getByLabelText(/ISP 用户名/i) as HTMLInputElement;
    const passwordInput = screen.getByLabelText(/ISP 密码/i) as HTMLInputElement;
    fireEvent.change(usernameInput, { target: { value: "testuser" } });
    fireEvent.change(passwordInput, { target: { value: "testpass" } });

    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(screen.getByText("Cascade Engine 已启动，系统代理已接管")).toBeDefined();
      expect(screen.getByRole("button", { name: /停止 Cascade/i })).toBeDefined();
    });
  });

  it("handles successful cascade engine stop", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce("Cascade Engine 已启动，系统代理已接管")
      .mockResolvedValueOnce("Cascade Engine 已停止，系统代理已恢复");

    render(<App />);

    const usernameInput = screen.getByLabelText(/ISP 用户名/i) as HTMLInputElement;
    const passwordInput = screen.getByLabelText(/ISP 密码/i) as HTMLInputElement;
    fireEvent.change(usernameInput, { target: { value: "testuser" } });
    fireEvent.change(passwordInput, { target: { value: "testpass" } });

    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /停止 Cascade/i })).toBeDefined();
    });

    const stopButton = screen.getByRole("button", { name: /停止 Cascade/i });
    fireEvent.click(stopButton);

    await waitFor(() => {
      expect(screen.getByText("Cascade Engine 已停止，系统代理已恢复")).toBeDefined();
      expect(screen.getByRole("button", { name: /启动 Cascade/i })).toBeDefined();
    });
  });

  it("handles error during cascade engine start", async () => {
    const errorMessage = "启动失败：端口 10808 已被占用";
    vi.mocked(invoke).mockRejectedValueOnce(errorMessage);

    render(<App />);

    const usernameInput = screen.getByLabelText(/ISP 用户名/i) as HTMLInputElement;
    const passwordInput = screen.getByLabelText(/ISP 密码/i) as HTMLInputElement;
    fireEvent.change(usernameInput, { target: { value: "testuser" } });
    fireEvent.change(passwordInput, { target: { value: "testpass" } });

    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(screen.getByText(errorMessage)).toBeDefined();
    });
  });
});
