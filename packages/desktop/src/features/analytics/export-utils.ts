// features/analytics/export-utils.ts - Export analytics data to CSV and PNG

import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile, writeFile } from "@tauri-apps/plugin-fs";
import type {
  AnalyticsDashboardData,
  FocusTrendDataPoint,
  SessionCompletionData,
  ProductivityScorePoint,
  TimeOfDayDistribution,
} from "@focusflow/types";

// CSV Export Functions

function escapeCSV(value: string | number | null | undefined): string {
  if (value === null || value === undefined) return "";
  const stringValue = String(value);
  if (stringValue.includes(",") || stringValue.includes('"') || stringValue.includes("\n")) {
    return `"${stringValue.replace(/"/g, '""')}"`;
  }
  return stringValue;
}

function arrayToCSV(headers: string[], rows: (string | number | null | undefined)[][]): string {
  const csvHeaders = headers.map(escapeCSV).join(",");
  const csvRows = rows.map((row) => row.map(escapeCSV).join(",")).join("\n");
  return `${csvHeaders}\n${csvRows}`;
}

function focusTrendToCSV(data: readonly FocusTrendDataPoint[]): string {
  const headers = ["Date", "Day of Week", "Focus Minutes", "Sessions", "Completion Rate (%)"];
  const rows = data.map((point) => [
    point.date,
    point.dayOfWeek,
    point.focusMinutes,
    point.sessions,
    point.completionRate,
  ]);
  return arrayToCSV(headers, rows);
}

function sessionCompletionToCSV(data: readonly SessionCompletionData[]): string {
  const headers = ["Date", "Completed", "Abandoned", "Total", "Completion Rate (%)"];
  const rows = data.map((point) => [
    point.date,
    point.completed,
    point.abandoned,
    point.total,
    point.rate,
  ]);
  return arrayToCSV(headers, rows);
}

function productivityScoresToCSV(data: readonly ProductivityScorePoint[]): string {
  const headers = ["Date", "Productivity Score", "Focus Minutes", "Distractions Blocked"];
  const rows = data.map((point) => [
    point.date,
    point.score,
    point.focusMinutes,
    point.distractionsBlocked,
  ]);
  return arrayToCSV(headers, rows);
}

function timeOfDayToCSV(data: readonly TimeOfDayDistribution[]): string {
  const headers = ["Hour", "Time", "Focus Minutes", "Sessions", "Percentage"];
  const rows = data.map((point) => [
    point.hour,
    point.label,
    point.focusMinutes,
    point.sessions,
    point.percentage,
  ]);
  return arrayToCSV(headers, rows);
}

function summaryToCSV(analytics: AnalyticsDashboardData): string {
  const headers = ["Metric", "Value"];
  const rows = [
    ["Date Range", `${analytics.dateRange.startDate} to ${analytics.dateRange.endDate}`],
    ["Total Focus Time (minutes)", analytics.summary.totalFocusMinutes],
    ["Total Focus Time (hours)", (analytics.summary.totalFocusMinutes / 60).toFixed(2)],
    ["Total Sessions", analytics.summary.totalSessions],
    ["Average Session Length (minutes)", analytics.summary.averageSessionLength],
    ["Completion Rate (%)", analytics.summary.completionRate],
    ["Average Productivity Score", analytics.summary.averageProductivityScore],
    ["Current Streak (days)", analytics.summary.currentStreak],
    ["Longest Streak (days)", analytics.summary.longestStreak],
    ["Most Productive Day", analytics.summary.mostProductiveDay || "N/A"],
    [
      "Most Productive Hour",
      analytics.summary.mostProductiveHour !== null
        ? formatHour(analytics.summary.mostProductiveHour)
        : "N/A",
    ],
    ["Total Distractions Blocked", analytics.summary.totalDistractionsBlocked],
  ];
  return arrayToCSV(headers, rows);
}

function formatHour(hour: number): string {
  if (hour === 0) return "12 AM";
  if (hour < 12) return `${hour} AM`;
  if (hour === 12) return "12 PM";
  return `${hour - 12} PM`;
}

export async function exportToCSV(analytics: AnalyticsDashboardData): Promise<void> {
  try {
    // Combine all CSV sections
    const sections = [
      "=== SUMMARY ===\n" + summaryToCSV(analytics),
      "\n\n=== FOCUS TREND ===\n" + focusTrendToCSV(analytics.focusTrend),
      "\n\n=== SESSION COMPLETION ===\n" + sessionCompletionToCSV(analytics.sessionCompletion),
      "\n\n=== PRODUCTIVITY SCORES ===\n" + productivityScoresToCSV(analytics.productivityScores),
      "\n\n=== TIME OF DAY DISTRIBUTION ===\n" + timeOfDayToCSV(analytics.timeOfDay),
    ];

    const csvContent = sections.join("");

    // Show save dialog
    const filePath = await save({
      title: "Export Analytics Data",
      defaultPath: `focus-analytics-${analytics.dateRange.startDate}-to-${analytics.dateRange.endDate}.csv`,
      filters: [
        {
          name: "CSV",
          extensions: ["csv"],
        },
      ],
    });

    if (filePath) {
      await writeTextFile(filePath, csvContent);
      return;
    }
  } catch (error) {
    console.error("Failed to export CSV:", error);
    throw new Error(
      `Failed to export CSV: ${error instanceof Error ? error.message : "Unknown error"}`
    );
  }
}

// PNG Export Functions

async function captureChartAsPNG(elementId: string): Promise<Blob | null> {
  const element = document.getElementById(elementId);
  if (!element) {
    console.error(`Element with id "${elementId}" not found`);
    return null;
  }

  // Dynamically import html2canvas only when needed
  const html2canvas = (await import("html2canvas")).default;

  const canvas = await html2canvas(element, {
    backgroundColor: getComputedStyle(element).backgroundColor || "#ffffff",
    scale: 2, // Higher quality
    logging: false,
    useCORS: true,
  });

  return new Promise((resolve) => {
    canvas.toBlob((blob) => resolve(blob), "image/png");
  });
}

async function captureDashboardAsPNG(): Promise<Blob | null> {
  // Find the main dashboard container
  const dashboard = document.querySelector(
    '[data-export-container="analytics-dashboard"]'
  ) as HTMLElement;
  if (!dashboard) {
    console.error("Dashboard container not found");
    return null;
  }

  // Dynamically import html2canvas
  const html2canvas = (await import("html2canvas")).default;

  const canvas = await html2canvas(dashboard, {
    backgroundColor: getComputedStyle(dashboard).backgroundColor || "#ffffff",
    scale: 2,
    logging: false,
    useCORS: true,
    windowWidth: dashboard.scrollWidth,
    windowHeight: dashboard.scrollHeight,
  });

  return new Promise((resolve) => {
    canvas.toBlob((blob) => resolve(blob), "image/png");
  });
}

export async function exportToPNG(analytics: AnalyticsDashboardData): Promise<void> {
  try {
    const blob = await captureDashboardAsPNG();
    if (!blob) {
      throw new Error("Failed to capture dashboard as image");
    }

    // Convert blob to ArrayBuffer
    const arrayBuffer = await blob.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    // Show save dialog
    const filePath = await save({
      title: "Export Analytics Dashboard",
      defaultPath: `focus-analytics-${analytics.dateRange.startDate}-to-${analytics.dateRange.endDate}.png`,
      filters: [
        {
          name: "PNG",
          extensions: ["png"],
        },
      ],
    });

    if (filePath) {
      await writeFile(filePath, uint8Array);
      return;
    }
  } catch (error) {
    console.error("Failed to export PNG:", error);
    throw new Error(
      `Failed to export PNG: ${error instanceof Error ? error.message : "Unknown error"}`
    );
  }
}

export async function exportChartToPNG(chartId: string, fileName: string): Promise<void> {
  try {
    const blob = await captureChartAsPNG(chartId);
    if (!blob) {
      throw new Error(`Failed to capture chart "${chartId}" as image`);
    }

    // Convert blob to ArrayBuffer
    const arrayBuffer = await blob.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    // Show save dialog
    const filePath = await save({
      title: "Export Chart",
      defaultPath: fileName,
      filters: [
        {
          name: "PNG",
          extensions: ["png"],
        },
      ],
    });

    if (filePath) {
      await writeFile(filePath, uint8Array);
      return;
    }
  } catch (error) {
    console.error("Failed to export chart PNG:", error);
    throw new Error(
      `Failed to export chart PNG: ${error instanceof Error ? error.message : "Unknown error"}`
    );
  }
}
