# Analytics Export Functionality

This document describes the analytics export feature implementation.

## Features

### CSV Export
- Exports all analytics data including:
  - Summary statistics (total focus time, sessions, completion rate, etc.)
  - Focus trend data (daily focus minutes, sessions, completion rates)
  - Session completion data (completed vs abandoned sessions)
  - Productivity scores over time
  - Time of day distribution (which hours are most productive)

### PNG Export
- Captures the entire analytics dashboard as a high-resolution PNG image
- Uses html2canvas library for rendering
- Preserves colors and styling from the dashboard

## Usage

From the Analytics Dashboard, click the "Export" dropdown button and select:
- **Export as CSV** - Downloads all analytics data as a CSV file
- **Export as PNG** - Downloads a screenshot of the entire dashboard

## Implementation Details

### Files Modified
- `/src/features/analytics/analytics-dashboard.tsx` - Added export buttons and handlers
- `/src/features/analytics/export-utils.ts` - Export utility functions (NEW)
- `/src-tauri/src/lib.rs` - Registered dialog plugin
- `/src-tauri/Cargo.toml` - Added dialog plugin dependency
- `/src-tauri/tauri.conf.json` - Added file system and dialog permissions
- `/src-tauri/capabilities/default.json` - Added dialog and fs permissions

### Dependencies
- `html2canvas@^1.4.1` - For capturing DOM elements as images
- `@tauri-apps/plugin-dialog` - For save file dialogs
- `@tauri-apps/plugin-fs` - For file system operations

### Permissions
The app requires the following Tauri permissions:
- `dialog:default` - Access to file dialogs
- `dialog:allow-save` - Save file dialog
- `fs:allow-write-file` - Write files to disk
- `fs:allow-write-text-file` - Write text files (CSV)
- File system scopes: `$DOWNLOAD/**`, `$DOCUMENT/**`, `$DESKTOP/**`

## CSV Format

The CSV export includes multiple sections:

```csv
=== SUMMARY ===
Metric,Value
Date Range,2024-01-01 to 2024-01-31
Total Focus Time (minutes),1234
...

=== FOCUS TREND ===
Date,Day of Week,Focus Minutes,Sessions,Completion Rate (%)
2024-01-01,Mon,120,5,80
...

=== SESSION COMPLETION ===
Date,Completed,Abandoned,Total,Completion Rate (%)
...
```

## PNG Export

The PNG export:
1. Finds the dashboard container using `data-export-container="analytics-dashboard"` attribute
2. Renders it to a canvas using html2canvas at 2x scale for quality
3. Converts to PNG blob
4. Saves via Tauri dialog

## Error Handling

- Toast notifications for success/failure
- Loading state during export
- Disabled export button when no data is available
- User-friendly error messages

## Future Enhancements

Potential improvements:
- Add individual chart export (export single charts)
- PDF export with formatted report
- Scheduled automated exports
- Email export functionality
- Custom date range selection for exports
