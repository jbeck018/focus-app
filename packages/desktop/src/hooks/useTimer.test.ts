// tests for useTimer hook

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTimer, formatTime, formatDuration } from "./useTimer";

describe("formatTime", () => {
  it("formats seconds as MM:SS", () => {
    expect(formatTime(0)).toBe("00:00");
    expect(formatTime(59)).toBe("00:59");
    expect(formatTime(60)).toBe("01:00");
    expect(formatTime(125)).toBe("02:05");
  });

  it("formats with hours when >= 1 hour", () => {
    expect(formatTime(3600)).toBe("01:00:00");
    expect(formatTime(3661)).toBe("01:01:01");
    expect(formatTime(7200)).toBe("02:00:00");
  });
});

describe("formatDuration", () => {
  it("formats seconds as human-readable duration", () => {
    expect(formatDuration(0)).toBe("0 minutes");
    expect(formatDuration(59)).toBe("0 minutes");
    expect(formatDuration(60)).toBe("1 minute");
    expect(formatDuration(120)).toBe("2 minutes");
  });

  it("formats with hours when >= 1 hour", () => {
    expect(formatDuration(3600)).toBe("1h 0m");
    expect(formatDuration(3660)).toBe("1h 1m");
    expect(formatDuration(7200)).toBe("2h 0m");
  });
});

describe("useTimer", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("initializes with 0 seconds and not running", () => {
    const { result } = renderHook(() => useTimer({ targetSeconds: 60 }));

    expect(result.current.seconds).toBe(0);
    expect(result.current.isRunning).toBe(false);
  });

  it("starts and increments timer", () => {
    const { result } = renderHook(() => useTimer({ targetSeconds: 60 }));

    act(() => {
      result.current.start();
    });

    expect(result.current.isRunning).toBe(true);

    act(() => {
      vi.advanceTimersByTime(1000);
    });

    expect(result.current.seconds).toBe(1);
  });

  it("pauses and resumes timer", () => {
    const { result } = renderHook(() => useTimer({ targetSeconds: 60 }));

    act(() => {
      result.current.start();
    });

    act(() => {
      vi.advanceTimersByTime(3000);
    });

    expect(result.current.seconds).toBe(3);

    act(() => {
      result.current.toggle();
    });

    expect(result.current.isRunning).toBe(false);

    act(() => {
      vi.advanceTimersByTime(2000);
    });

    // Should not have advanced while paused
    expect(result.current.seconds).toBe(3);

    act(() => {
      result.current.toggle();
    });

    expect(result.current.isRunning).toBe(true);
  });

  it("resets timer to 0", () => {
    const { result } = renderHook(() => useTimer({ targetSeconds: 60 }));

    act(() => {
      result.current.start();
    });

    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(result.current.seconds).toBe(5);

    act(() => {
      result.current.reset();
    });

    expect(result.current.seconds).toBe(0);
    expect(result.current.isRunning).toBe(false);
  });

  it("calls onComplete when reaching target", () => {
    const onComplete = vi.fn();
    const { result } = renderHook(() => useTimer({ targetSeconds: 3, onComplete }));

    act(() => {
      result.current.start();
    });

    act(() => {
      vi.advanceTimersByTime(3000);
    });

    expect(onComplete).toHaveBeenCalledTimes(1);
    expect(result.current.isRunning).toBe(false);
  });

  it("calls onTick with current seconds", () => {
    const onTick = vi.fn();
    const { result } = renderHook(() => useTimer({ targetSeconds: 60, onTick }));

    act(() => {
      result.current.start();
    });

    act(() => {
      vi.advanceTimersByTime(3000);
    });

    expect(onTick).toHaveBeenCalledWith(1);
    expect(onTick).toHaveBeenCalledWith(2);
    expect(onTick).toHaveBeenCalledWith(3);
  });
});
