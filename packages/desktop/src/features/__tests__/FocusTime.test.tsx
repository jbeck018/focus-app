// Tests for FocusTime component
// Covers: rendering, interactions, state changes, accessibility

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React, { type ReactNode } from "react";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]): unknown => mockInvoke(...args),
}));

// Mock Tauri events
const mockListen = vi.fn();
vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]): unknown => mockListen(...args),
}));

// Test data
const mockFocusTimeState = {
  active: false,
  eventId: null,
  eventTitle: null,
  startedAt: null,
  endsAt: null,
  allowedApps: [],
  endedEarly: false,
};

const mockActiveFocusTime = {
  active: true,
  eventId: "event-123",
  eventTitle: "Coding Session",
  startedAt: new Date().toISOString(),
  endsAt: new Date(Date.now() + 3600000).toISOString(),
  allowedApps: ["Code", "Terminal", "Notion"],
  endedEarly: false,
};

const mockFocusTimeEvents = [
  {
    id: "event-123",
    title: "[Focus] Coding Session",
    cleanTitle: "Coding Session",
    description: "@coding, @terminal",
    startTime: new Date(Date.now() - 1800000).toISOString(),
    endTime: new Date(Date.now() + 1800000).toISOString(),
    durationMinutes: 60,
    allowedApps: ["Code", "Terminal"],
    isActive: true,
    isUpcoming: false,
    source: "calendar",
  },
  {
    id: "event-456",
    title: "Deep Work - Writing",
    cleanTitle: "Deep Work - Writing",
    description: "@writing",
    startTime: new Date(Date.now() + 1800000).toISOString(),
    endTime: new Date(Date.now() + 5400000).toISOString(),
    durationMinutes: 60,
    allowedApps: ["Notion", "Obsidian"],
    isActive: false,
    isUpcoming: true,
    source: "calendar",
  },
];

const mockAppCategories = [
  {
    id: "@coding",
    name: "Coding",
    description: "Code editors and IDEs",
    exampleApps: ["VS Code", "Xcode", "IntelliJ"],
  },
  {
    id: "@terminal",
    name: "Terminal",
    description: "Terminal emulators",
    exampleApps: ["Terminal", "iTerm2", "Warp"],
  },
  {
    id: "@writing",
    name: "Writing",
    description: "Writing apps",
    exampleApps: ["Notion", "Obsidian", "Word"],
  },
];

// Simple Focus Time component for testing
interface FocusTimeProps {
  state: typeof mockFocusTimeState | typeof mockActiveFocusTime;
  events?: typeof mockFocusTimeEvents;
  onStart?: (eventId: string) => Promise<void>;
  onEnd?: (early: boolean) => Promise<void>;
  onAddApp?: (app: string) => Promise<void>;
  onRemoveApp?: (app: string) => Promise<void>;
}

function FocusTimeDisplay({
  state,
  events = [],
  onStart,
  onEnd,
  onAddApp,
  onRemoveApp,
}: FocusTimeProps) {
  const [isLoading, setIsLoading] = React.useState(false);
  const [newApp, setNewApp] = React.useState("");

  const handleStart = async (eventId: string) => {
    if (!onStart) return;
    setIsLoading(true);
    try {
      await onStart(eventId);
    } finally {
      setIsLoading(false);
    }
  };

  const handleEnd = async (early: boolean) => {
    if (!onEnd) return;
    setIsLoading(true);
    try {
      await onEnd(early);
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddApp = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!onAddApp || !newApp.trim()) return;
    await onAddApp(newApp.trim());
    setNewApp("");
  };

  if (state.active) {
    return (
      <div data-testid="focus-time-active">
        <h2>Focus Time Active</h2>
        <p data-testid="event-title">{state.eventTitle}</p>
        <div data-testid="remaining-time" aria-live="polite">
          Time remaining:{" "}
          {state.endsAt
            ? Math.max(0, Math.floor((new Date(state.endsAt).getTime() - Date.now()) / 60000))
            : 0}{" "}
          minutes
        </div>

        <section aria-label="Allowed Apps">
          <h3>Allowed Apps</h3>
          <ul data-testid="allowed-apps-list">
            {state.allowedApps.map((app) => (
              <li key={app}>
                {app}
                <button
                  onClick={() => onRemoveApp?.(app)}
                  aria-label={`Remove ${app} from allowed apps`}
                >
                  Remove
                </button>
              </li>
            ))}
          </ul>

          <form onSubmit={handleAddApp}>
            <input
              type="text"
              value={newApp}
              onChange={(e) => setNewApp(e.target.value)}
              placeholder="Add app..."
              aria-label="New app name"
            />
            <button type="submit">Add App</button>
          </form>
        </section>

        <div>
          <button
            onClick={() => handleEnd(true)}
            disabled={isLoading}
            data-testid="end-early-button"
          >
            End Early
          </button>
        </div>
      </div>
    );
  }

  return (
    <div data-testid="focus-time-inactive">
      <h2>Focus Time</h2>
      <p>No Focus Time active</p>

      {events.length > 0 && (
        <section aria-label="Focus Time Events">
          <h3>Upcoming Focus Time</h3>
          <ul data-testid="events-list">
            {events.map((event) => (
              <li key={event.id} data-testid={`event-${event.id}`}>
                <span>{event.cleanTitle}</span>
                <span>{event.isActive ? " (Active)" : event.isUpcoming ? " (Upcoming)" : ""}</span>
                <button
                  onClick={() => handleStart(event.id)}
                  disabled={isLoading}
                  aria-label={`Start ${event.cleanTitle}`}
                >
                  {event.isActive ? "Resume" : "Start"}
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}

// Category selector component
interface CategorySelectorProps {
  categories: typeof mockAppCategories;
  selected: string[];
  onChange: (selected: string[]) => void;
}

function CategorySelector({ categories, selected, onChange }: CategorySelectorProps) {
  const handleToggle = (categoryId: string) => {
    if (selected.includes(categoryId)) {
      onChange(selected.filter((id) => id !== categoryId));
    } else {
      onChange([...selected, categoryId]);
    }
  };

  return (
    <div data-testid="category-selector" role="group" aria-label="App Categories">
      {categories.map((category) => (
        <label key={category.id}>
          <input
            type="checkbox"
            checked={selected.includes(category.id)}
            onChange={() => handleToggle(category.id)}
            aria-describedby={`${category.id}-description`}
          />
          {category.name}
          <span id={`${category.id}-description`} className="sr-only">
            {category.description}
          </span>
        </label>
      ))}
    </div>
  );
}

// Test wrapper
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return React.createElement(QueryClientProvider, { client: queryClient }, children);
  };
}

describe("FocusTimeDisplay", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe("inactive state", () => {
    it("should render inactive state message", () => {
      render(<FocusTimeDisplay state={mockFocusTimeState} />, { wrapper: createWrapper() });

      expect(screen.getByTestId("focus-time-inactive")).toBeInTheDocument();
      expect(screen.getByText("No Focus Time active")).toBeInTheDocument();
    });

    it("should display upcoming Focus Time events", () => {
      render(<FocusTimeDisplay state={mockFocusTimeState} events={mockFocusTimeEvents} />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByTestId("events-list")).toBeInTheDocument();
      expect(screen.getByText("Coding Session")).toBeInTheDocument();
      expect(screen.getByText("Deep Work - Writing")).toBeInTheDocument();
    });

    it("should allow starting Focus Time from event", async () => {
      const onStart = vi.fn().mockResolvedValue(undefined);
      const user = userEvent.setup();

      render(
        <FocusTimeDisplay
          state={mockFocusTimeState}
          events={mockFocusTimeEvents}
          onStart={onStart}
        />,
        { wrapper: createWrapper() }
      );

      const startButtons = screen.getAllByRole("button", { name: /Start|Resume/i });
      await user.click(startButtons[0]);

      expect(onStart).toHaveBeenCalledWith("event-123");
    });
  });

  describe("active state", () => {
    it("should render active Focus Time display", () => {
      render(<FocusTimeDisplay state={mockActiveFocusTime} />, { wrapper: createWrapper() });

      expect(screen.getByTestId("focus-time-active")).toBeInTheDocument();
      expect(screen.getByTestId("event-title")).toHaveTextContent("Coding Session");
    });

    it("should display remaining time", () => {
      render(<FocusTimeDisplay state={mockActiveFocusTime} />, { wrapper: createWrapper() });

      const timeDisplay = screen.getByTestId("remaining-time");
      expect(timeDisplay).toBeInTheDocument();
      expect(timeDisplay).toHaveTextContent(/minutes/);
    });

    it("should display allowed apps list", () => {
      render(<FocusTimeDisplay state={mockActiveFocusTime} />, { wrapper: createWrapper() });

      const appsList = screen.getByTestId("allowed-apps-list");
      expect(appsList).toBeInTheDocument();
      expect(within(appsList).getByText("Code")).toBeInTheDocument();
      expect(within(appsList).getByText("Terminal")).toBeInTheDocument();
      expect(within(appsList).getByText("Notion")).toBeInTheDocument();
    });

    it("should allow ending Focus Time early", async () => {
      const onEnd = vi.fn().mockResolvedValue(undefined);
      const user = userEvent.setup();

      render(<FocusTimeDisplay state={mockActiveFocusTime} onEnd={onEnd} />, {
        wrapper: createWrapper(),
      });

      const endButton = screen.getByTestId("end-early-button");
      await user.click(endButton);

      expect(onEnd).toHaveBeenCalledWith(true);
    });

    it("should allow adding an allowed app", async () => {
      const onAddApp = vi.fn().mockResolvedValue(undefined);
      const user = userEvent.setup();

      render(<FocusTimeDisplay state={mockActiveFocusTime} onAddApp={onAddApp} />, {
        wrapper: createWrapper(),
      });

      const input = screen.getByPlaceholderText("Add app...");
      const addButton = screen.getByRole("button", { name: "Add App" });

      await user.type(input, "Slack");
      await user.click(addButton);

      expect(onAddApp).toHaveBeenCalledWith("Slack");
    });

    it("should allow removing an allowed app", async () => {
      const onRemoveApp = vi.fn().mockResolvedValue(undefined);
      const user = userEvent.setup();

      render(<FocusTimeDisplay state={mockActiveFocusTime} onRemoveApp={onRemoveApp} />, {
        wrapper: createWrapper(),
      });

      const removeButton = screen.getByRole("button", {
        name: "Remove Code from allowed apps",
      });
      await user.click(removeButton);

      expect(onRemoveApp).toHaveBeenCalledWith("Code");
    });
  });

  describe("loading states", () => {
    it("should disable buttons during loading", async () => {
      const onEnd = vi
        .fn()
        .mockImplementation(() => new Promise((resolve) => setTimeout(resolve, 1000)));
      const user = userEvent.setup();

      render(<FocusTimeDisplay state={mockActiveFocusTime} onEnd={onEnd} />, {
        wrapper: createWrapper(),
      });

      const endButton = screen.getByTestId("end-early-button");
      await user.click(endButton);

      expect(endButton).toBeDisabled();
    });
  });
});

describe("CategorySelector", () => {
  it("should render all categories", () => {
    render(<CategorySelector categories={mockAppCategories} selected={[]} onChange={vi.fn()} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByRole("checkbox", { name: /coding/i })).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: /terminal/i })).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: /writing/i })).toBeInTheDocument();
  });

  it("should show selected categories as checked", () => {
    render(
      <CategorySelector
        categories={mockAppCategories}
        selected={["@coding", "@terminal"]}
        onChange={vi.fn()}
      />,
      { wrapper: createWrapper() }
    );

    expect(screen.getByRole("checkbox", { name: /coding/i })).toBeChecked();
    expect(screen.getByRole("checkbox", { name: /terminal/i })).toBeChecked();
    expect(screen.getByRole("checkbox", { name: /writing/i })).not.toBeChecked();
  });

  it("should call onChange when category is selected", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();

    render(
      <CategorySelector
        categories={mockAppCategories}
        selected={["@coding"]}
        onChange={onChange}
      />,
      { wrapper: createWrapper() }
    );

    // Select another category
    const terminalCheckbox = screen.getByRole("checkbox", { name: /terminal/i });
    await user.click(terminalCheckbox);
    expect(onChange).toHaveBeenCalledWith(["@coding", "@terminal"]);
  });

  it("should call onChange when category is deselected", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();

    render(
      <CategorySelector
        categories={mockAppCategories}
        selected={["@coding", "@terminal"]}
        onChange={onChange}
      />,
      { wrapper: createWrapper() }
    );

    const codingCheckbox = screen.getByRole("checkbox", { name: /coding/i });
    await user.click(codingCheckbox);
    expect(onChange).toHaveBeenCalledWith(["@terminal"]);
  });
});

describe("accessibility", () => {
  it("should have accessible labels for interactive elements", () => {
    render(<FocusTimeDisplay state={mockActiveFocusTime} onRemoveApp={vi.fn()} />, {
      wrapper: createWrapper(),
    });

    // Check remove buttons have accessible names
    const removeButtons = screen.getAllByRole("button", { name: /Remove .* from allowed apps/ });
    expect(removeButtons.length).toBeGreaterThan(0);
  });

  it("should have live region for time updates", () => {
    render(<FocusTimeDisplay state={mockActiveFocusTime} />, { wrapper: createWrapper() });

    const timeDisplay = screen.getByTestId("remaining-time");
    expect(timeDisplay).toHaveAttribute("aria-live", "polite");
  });

  it("should have proper section labeling", () => {
    render(<FocusTimeDisplay state={mockActiveFocusTime} />, { wrapper: createWrapper() });

    expect(screen.getByRole("region", { name: "Allowed Apps" })).toBeInTheDocument();
  });

  it("should be keyboard navigable", async () => {
    const user = userEvent.setup();

    render(<FocusTimeDisplay state={mockActiveFocusTime} onEnd={vi.fn()} />, {
      wrapper: createWrapper(),
    });

    // Tab through interactive elements
    await user.tab();
    expect(document.activeElement).toBeInstanceOf(HTMLButtonElement);
  });
});

describe("edge cases", () => {
  it("should handle empty allowed apps list", () => {
    const stateWithNoApps = {
      ...mockActiveFocusTime,
      allowedApps: [],
    };

    render(<FocusTimeDisplay state={stateWithNoApps} />, { wrapper: createWrapper() });

    const appsList = screen.getByTestId("allowed-apps-list");
    expect(appsList).toBeEmptyDOMElement();
  });

  it("should handle empty events list", () => {
    render(<FocusTimeDisplay state={mockFocusTimeState} events={[]} />, {
      wrapper: createWrapper(),
    });

    expect(screen.queryByTestId("events-list")).not.toBeInTheDocument();
  });

  it("should handle null event title gracefully", () => {
    const stateWithNullTitle = {
      ...mockActiveFocusTime,
      eventTitle: null as unknown as string,
    };

    render(<FocusTimeDisplay state={stateWithNullTitle} />, { wrapper: createWrapper() });

    expect(screen.getByTestId("focus-time-active")).toBeInTheDocument();
  });

  it("should prevent adding empty app names", async () => {
    const onAddApp = vi.fn();
    const user = userEvent.setup();

    render(<FocusTimeDisplay state={mockActiveFocusTime} onAddApp={onAddApp} />, {
      wrapper: createWrapper(),
    });

    const addButton = screen.getByRole("button", { name: "Add App" });
    await user.click(addButton);

    // Should not call onAddApp with empty string
    expect(onAddApp).not.toHaveBeenCalled();
  });

  it("should trim whitespace from app names", async () => {
    const onAddApp = vi.fn().mockResolvedValue(undefined);
    const user = userEvent.setup();

    render(<FocusTimeDisplay state={mockActiveFocusTime} onAddApp={onAddApp} />, {
      wrapper: createWrapper(),
    });

    const input = screen.getByPlaceholderText("Add app...");
    await user.type(input, "  Slack  ");
    await user.click(screen.getByRole("button", { name: "Add App" }));

    expect(onAddApp).toHaveBeenCalledWith("Slack");
  });
});

describe("Focus Time event parsing", () => {
  interface ParseResult {
    isFocusTime: boolean;
    allowedApps: string[];
    categories: string[];
  }

  // Utility function simulating frontend parsing
  function parseFocusTimeFromEvent(title: string, description?: string): ParseResult {
    const focusKeywords = [
      "focus time",
      "focus block",
      "deep work",
      "focus session",
      "no meetings",
      "heads down",
      "do not disturb",
      "dnd",
      "coding time",
      "writing time",
    ];

    const titleLower = title.toLowerCase();
    const isFocusTime = focusKeywords.some((kw) => titleLower.includes(kw));

    if (!isFocusTime || !description) {
      return { isFocusTime, allowedApps: [], categories: [] };
    }

    // Parse categories and apps from description
    const categories: string[] = [];
    const allowedApps: string[] = [];

    const items = description
      .split(/[,\n]/)
      .map((s) => s.trim())
      .filter(Boolean);
    for (const item of items) {
      if (item.startsWith("@")) {
        categories.push(item);
      } else if (!item.toLowerCase().startsWith("allowed:")) {
        allowedApps.push(item);
      }
    }

    return { isFocusTime, allowedApps, categories };
  }

  it("should detect focus time from title", () => {
    expect(parseFocusTimeFromEvent("Focus Time").isFocusTime).toBe(true);
    expect(parseFocusTimeFromEvent("Deep Work").isFocusTime).toBe(true);
    expect(parseFocusTimeFromEvent("Team Meeting").isFocusTime).toBe(false);
  });

  it("should parse categories from description", () => {
    const result = parseFocusTimeFromEvent("Focus Time", "@coding, @terminal");
    expect(result.categories).toContain("@coding");
    expect(result.categories).toContain("@terminal");
  });

  it("should parse direct apps from description", () => {
    const result = parseFocusTimeFromEvent("Focus Time", "@coding, Notion, Obsidian");
    expect(result.allowedApps).toContain("Notion");
    expect(result.allowedApps).toContain("Obsidian");
  });

  it("should handle multiline descriptions", () => {
    const desc = `Allowed:
@coding
@terminal
Notion`;
    const result = parseFocusTimeFromEvent("Focus Time", desc);
    expect(result.categories).toContain("@coding");
    expect(result.categories).toContain("@terminal");
    expect(result.allowedApps).toContain("Notion");
  });
});
