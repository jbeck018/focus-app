# Interactive Analytics Dashboard

Comprehensive analytics dashboard for FocusFlow with interactive Recharts visualizations.

## Overview

The analytics dashboard provides deep insights into productivity patterns through six interactive chart types:

1. **Focus Trend Chart** - Line/Area chart showing daily/weekly/monthly focus time trends
2. **Session Completion Chart** - Stacked bar chart showing completed vs abandoned sessions
3. **Productivity Chart** - Area chart tracking productivity score over time
4. **Time of Day Chart** - Radial chart analyzing productivity by time periods
5. **Distractions Chart** - Horizontal bar chart showing top blocked distractions
6. **Calendar Heatmap** - 365-day GitHub-style heatmap view

## Architecture

### File Structure

```
features/analytics/
├── analytics-dashboard.tsx       # Main container component
├── charts/
│   ├── index.ts                 # Barrel export for all charts
│   ├── focus-trend-chart.tsx    # Focus time trend visualization
│   ├── session-completion-chart.tsx
│   ├── productivity-chart.tsx
│   ├── time-of-day-chart.tsx
│   ├── distractions-chart.tsx
│   └── calendar-heatmap.tsx
└── README.md

hooks/
└── use-analytics.ts              # Analytics data fetching & transformation

types/src/
└── analytics-extended.ts         # Extended analytics type definitions
```

## Features

### Interactive Capabilities

- **Date Range Filtering**: Today, Last 7/14/30 days, This/Last Month, This Year, Custom
- **Click-through Drill-down**: Click data points for detailed breakdowns
- **Export Options**: CSV and PNG export (to be implemented)
- **Responsive Design**: Adapts to all screen sizes
- **Dark Mode Support**: Full dark theme compatibility with custom chart colors

### Chart Details

#### 1. Focus Trend Chart
- Displays focus minutes and session count over time
- Dual Y-axis for minutes and sessions
- Line or Area variant
- Shows average daily focus time
- Click data points to see daily details

#### 2. Session Completion Chart
- Stacked bar chart comparing completed vs abandoned sessions
- Shows completion percentage
- Color-coded (green = completed, red = abandoned)
- Summary stats at bottom

#### 3. Productivity Chart
- Area chart with gradient fill
- Score range 0-100
- Reference line at target (60)
- Shows peak, low, and average scores
- Color-coded status labels (Excellent, Good, Fair, Needs Work)

#### 4. Time of Day Chart
- Radial bar chart showing productivity by time period
- Groups into Morning, Afternoon, Evening, Night
- Percentage distribution
- Shows most productive time period

#### 5. Distractions Chart
- Horizontal bar chart for easy reading
- Limited to top 10 distractions
- Distinguishes apps vs websites
- Shows trend indicators (up/down/stable)
- Empty state handling

#### 6. Calendar Heatmap
- 365-day activity view
- 5 intensity levels (0-4)
- Hover tooltips with daily details
- Shows active days, total focus, and best day
- Month and day labels

## Usage

### Basic Implementation

```typescript
import { AnalyticsDashboard } from "@/features/analytics";

function App() {
  return <AnalyticsDashboard />;
}
```

### Using Individual Charts

```typescript
import {
  FocusTrendChart,
  SessionCompletionChart,
  ProductivityChart
} from "@/features/analytics/charts";

function CustomDashboard() {
  const { data } = useAnalyticsDashboard(filters);

  return (
    <div>
      <FocusTrendChart
        data={data.focusTrend}
        variant="area"
        showSessions={true}
      />
      <SessionCompletionChart data={data.sessionCompletion} />
      <ProductivityChart data={data.productivityScores} />
    </div>
  );
}
```

### Custom Date Ranges

```typescript
import { useAnalyticsDashboard } from "@/hooks/use-analytics";
import type { ChartFilters } from "@focusflow/types";

const filters: ChartFilters = {
  dateRange: { preset: "last_30_days" },
  granularity: "daily",
  includeAbandoned: true,
};

const { data, isLoading } = useAnalyticsDashboard(filters);
```

## Data Flow

1. **User Selection** → Date range preset selected
2. **Hook Processing** → `useAnalyticsDashboard` calculates date range
3. **Tauri Backend** → `get_date_range_stats` fetches data from SQLite
4. **Data Transformation** → Hook transforms Rust response to chart-ready format
5. **Chart Rendering** → Recharts renders interactive visualizations

## Backend Integration

### Tauri Commands

```rust
// Get analytics for date range
#[tauri::command]
pub async fn get_date_range_stats(
    start_date: String,
    end_date: String,
    state: State<'_, AppState>,
) -> Result<Vec<DailyStatsResponse>>
```

### Response Format

```typescript
interface DailyStatsResponse {
  date: string;
  total_focus_minutes: number;
  total_break_minutes: number;
  sessions_completed: number;
  sessions_abandoned: number;
  productivity_score: number;
}
```

## Type Safety

All chart data uses branded types for safety:

```typescript
type DateString = string & { readonly __brand: "DateString" };
type Minutes = number & { readonly __brand: "Minutes" };
type Percentage = number & { readonly __brand: "Percentage" };
```

## Styling

### Chart Colors

Defined in `index.css`:
- `--chart-1`: Primary data series (focus time)
- `--chart-2`: Secondary series (sessions)
- `--chart-3`: Tertiary series
- `--chart-4`: Quaternary series

Colors automatically adapt to dark mode.

### Customization

All charts use shadcn/ui Card components for consistent styling:

```typescript
<Card>
  <CardHeader>
    <CardTitle>Chart Title</CardTitle>
  </CardHeader>
  <CardContent>
    {/* Recharts component */}
  </CardContent>
</Card>
```

## Performance

- **Lazy Loading**: Charts only render when visible
- **Memoization**: Data transformations cached with `useMemo`
- **Query Caching**: React Query caches for 5 minutes
- **Optimized Re-renders**: Only re-fetch when filters change

## Future Enhancements

- [ ] Export to CSV/PNG implementation
- [ ] Custom date range picker UI
- [ ] Real-time distraction blocking data
- [ ] Comparison mode (period over period)
- [ ] AI-powered insights generation
- [ ] Session type filtering
- [ ] Granularity switching (hourly/daily/weekly/monthly)
- [ ] Chart customization preferences
- [ ] Print-friendly layouts

## Dependencies

- **recharts**: ^3.6.0 - Chart library
- **@tanstack/react-query**: Data fetching and caching
- **lucide-react**: Icons
- **shadcn/ui**: UI components

## Testing

```bash
# Type checking
pnpm typecheck

# Run tests (when implemented)
pnpm test

# Build check
pnpm build
```

## Troubleshooting

### Charts not displaying
- Verify data is being fetched: Check React Query DevTools
- Ensure date range is valid
- Check browser console for errors

### Empty state
- Verify sessions exist in database
- Check date range includes data
- Ensure productivity scores are calculated

### Performance issues
- Reduce date range for large datasets
- Enable granularity switching
- Check for memory leaks in DevTools

## Contributing

When adding new chart types:

1. Create chart component in `charts/` directory
2. Add type definitions to `analytics-extended.ts`
3. Update `useAnalyticsDashboard` hook to provide data
4. Export from `charts/index.ts`
5. Add to dashboard layout
6. Update this README

## License

Part of FocusFlow - see main project LICENSE
