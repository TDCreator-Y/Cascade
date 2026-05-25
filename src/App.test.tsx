import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";

// Mock the Tauri invoke function
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock the Tauri store plugin
vi.mock("@tauri-apps/plugin-store", () => ({
  LazyStore: class {
    async get() { return null; }
    async set() { return undefined; }
    async save() { return undefined; }
  }
}));

describe("App Component", () => {
  it("renders the main interface correctly", () => {
    render(<App />);
    
    // Check if title is present
    expect(screen.getByText("Cascade Config")).toBeDefined();
    
    // Check if start button is present
    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    expect(startButton).toBeDefined();
  });

  it("handles successful cascade engine start and stop", async () => {
    // Setup mock to resolve successfully
    vi.mocked(invoke).mockResolvedValueOnce("Cascade Engine started successfully");
    
    render(<App />);
    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    
    // Click the button
    fireEvent.click(startButton);
    
    // Wait for the success message and stop button to appear
    await waitFor(() => {
      expect(screen.getByText("Cascade Engine started successfully")).toBeDefined();
      expect(screen.getByRole("button", { name: /停止 Cascade/i })).toBeDefined();
    });

    // Mock stop function
    vi.mocked(invoke).mockResolvedValueOnce("Cascade Engine stopped successfully");
    const stopButton = screen.getByRole("button", { name: /停止 Cascade/i });
    fireEvent.click(stopButton);

    await waitFor(() => {
      expect(screen.getByText("Cascade Engine 已停止，系统代理已恢复")).toBeDefined();
      expect(screen.getByRole("button", { name: /启动 Cascade/i })).toBeDefined();
    });
  });

  it("handles error during cascade engine start", async () => {
    // Setup mock to reject with an error
    const errorMessage = "Failed to bind port";
    vi.mocked(invoke).mockRejectedValueOnce(errorMessage);
    
    render(<App />);
    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    
    // Click the button
    fireEvent.click(startButton);
    
    // Wait for the error message to appear
    await waitFor(() => {
      expect(screen.getByText(errorMessage)).toBeDefined();
    });
  });
});