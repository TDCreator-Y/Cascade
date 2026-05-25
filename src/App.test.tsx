import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";

// Mock the Tauri invoke function
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("App Component", () => {
  it("renders the main interface correctly", () => {
    render(<App />);
    
    // Check if title is present
    expect(screen.getByText("Cascade")).toBeDefined();
    expect(screen.getByText("Advanced Network Proxy Engine")).toBeDefined();
    
    // Check if start button is present
    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    expect(startButton).toBeDefined();
  });

  it("handles successful cascade engine start", async () => {
    // Setup mock to resolve successfully
    vi.mocked(invoke).mockResolvedValueOnce("Cascade Engine started successfully");
    
    render(<App />);
    const startButton = screen.getByRole("button", { name: /启动 Cascade/i });
    
    // Click the button
    fireEvent.click(startButton);
    
    // Button should show loading state immediately
    expect(screen.getByText("Starting Engine...")).toBeDefined();
    
    // Wait for the success message to appear
    await waitFor(() => {
      expect(screen.getByText("Cascade Engine started successfully")).toBeDefined();
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