// Test setup file for Vitest

import "@testing-library/jest-dom";
import { vi } from "vitest";

// Mock Tauri API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock window.__TAURI__ for Tauri detection
Object.defineProperty(window, "__TAURI__", {
  value: {},
  writable: true,
});
