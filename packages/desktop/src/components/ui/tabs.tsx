/**
 * Tabs Component with Enhanced Accessibility
 *
 * WCAG 2.1 Level AA Compliance:
 * - 2.1.1 Keyboard: Arrow key navigation between tabs
 * - 2.4.3 Focus Order: Proper tab sequence
 * - 4.1.2 Name, Role, Value: ARIA tabs pattern
 *
 * Keyboard Navigation (provided by Radix UI):
 * - Left/Right Arrow: Navigate between tabs (horizontal)
 * - Home: Jump to first tab
 * - End: Jump to last tab
 * - Tab: Move focus to tab panel content
 * - Enter/Space: Activate focused tab
 *
 * ARIA Pattern: https://www.w3.org/WAI/ARIA/apg/patterns/tabs/
 */

import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";

import { cn } from "@/lib/utils";

function Tabs({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Root>) {
  return (
    <TabsPrimitive.Root
      data-slot="tabs"
      className={cn("flex flex-col gap-2", className)}
      {...props}
    />
  );
}

/**
 * TabsList Component
 * Implements ARIA tablist role with proper keyboard navigation
 */
function TabsList({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.List>) {
  return (
    <TabsPrimitive.List
      data-slot="tabs-list"
      role="tablist"
      className={cn(
        "bg-muted text-muted-foreground inline-flex h-9 w-fit items-center justify-center rounded-lg p-[3px]",
        className
      )}
      {...props}
    />
  );
}

/**
 * TabsTrigger Component
 * Individual tab with enhanced focus indicators
 * - Implements roving tabindex pattern
 * - Visible focus ring (WCAG 2.4.7)
 * - aria-selected state
 */
function TabsTrigger({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Trigger>) {
  return (
    <TabsPrimitive.Trigger
      data-slot="tabs-trigger"
      role="tab"
      className={cn(
        "data-[state=active]:bg-background dark:data-[state=active]:text-foreground focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:outline-ring dark:data-[state=active]:border-input dark:data-[state=active]:bg-input/30 text-foreground dark:text-muted-foreground inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 rounded-md border border-transparent px-2 py-1 text-sm font-medium whitespace-nowrap transition-[color,box-shadow] focus-visible:ring-[3px] focus-visible:outline-1 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:shadow-sm [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        className
      )}
      {...props}
    />
  );
}

/**
 * TabsContent Component
 * Tab panel content with proper ARIA attributes
 * - Implements tabpanel role
 * - Associated with corresponding tab via aria-labelledby
 */
function TabsContent({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Content>) {
  return (
    <TabsPrimitive.Content
      data-slot="tabs-content"
      role="tabpanel"
      tabIndex={0}
      className={cn("flex-1 outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 rounded-md", className)}
      {...props}
    />
  );
}

export { Tabs, TabsList, TabsTrigger, TabsContent };
