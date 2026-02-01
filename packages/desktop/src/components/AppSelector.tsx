// components/AppSelector.tsx - App selection component for Focus Time overrides

import { useState, useMemo } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  useFocusTimeCategories,
  useFocusTimeCommonApps,
  type CategoryInfo,
} from "@/hooks/useFocusTime";
import {
  Search,
  Check,
  Laptop,
  MessageSquare,
  Globe,
  Palette,
  FileText,
  Terminal,
  Music,
  PenTool,
  Loader2,
  HelpCircle,
} from "lucide-react";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";

interface AppSelectorProps {
  selectedApps: string[];
  onToggleApp: (appName: string) => void;
  onToggleCategory: (categoryId: string) => void;
  onClose: () => void;
}

// Map category IDs to icons
const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  "@coding": <Laptop className="h-4 w-4" />,
  "@communication": <MessageSquare className="h-4 w-4" />,
  "@browser": <Globe className="h-4 w-4" />,
  "@design": <Palette className="h-4 w-4" />,
  "@productivity": <FileText className="h-4 w-4" />,
  "@terminal": <Terminal className="h-4 w-4" />,
  "@music": <Music className="h-4 w-4" />,
  "@writing": <PenTool className="h-4 w-4" />,
};

export function AppSelector({
  selectedApps,
  onToggleApp,
  onToggleCategory,
  onClose,
}: AppSelectorProps) {
  const [searchQuery, setSearchQuery] = useState("");

  // Fetch categories and common apps from backend
  const { data: categories, isLoading: categoriesLoading } = useFocusTimeCategories();
  const { data: commonApps, isLoading: appsLoading } = useFocusTimeCommonApps();

  // Build list of all apps from backend data
  const allApps = useMemo(() => {
    if (!commonApps) return [];
    // Get unique app names from common apps
    const apps = new Set<string>();
    commonApps.forEach((app) => apps.add(app.name));
    return Array.from(apps).sort();
  }, [commonApps]);

  // Filter apps based on search
  const filteredApps = useMemo(() => {
    if (!searchQuery) return allApps;
    const query = searchQuery.toLowerCase();
    return allApps.filter((app) => app.toLowerCase().includes(query));
  }, [allApps, searchQuery]);

  // Check if category is fully selected based on example apps
  const isCategorySelected = (category: CategoryInfo) => {
    // Check if the category ID is in selected apps (user selected entire category)
    if (selectedApps.includes(category.id)) return true;
    // Check if all example apps are selected
    return category.exampleApps.every((app) => selectedApps.includes(app));
  };

  const isLoading = categoriesLoading || appsLoading;

  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
          <span className="text-sm text-muted-foreground">Loading apps...</span>
        </div>
        <div className="space-y-2">
          <Skeleton className="h-8 w-full" />
          <Skeleton className="h-8 w-3/4" />
          <Skeleton className="h-8 w-5/6" />
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <Search className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
        <Input
          placeholder="Search apps (e.g., Slack, Notion, VS Code)..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="flex-1"
          aria-label="Search applications"
        />
      </div>

      {/* Pro Tip Banner */}
      <div className="rounded-md bg-primary/10 border border-primary/20 p-3">
        <p className="text-xs text-muted-foreground flex items-start gap-2">
          <HelpCircle className="h-4 w-4 text-primary flex-shrink-0 mt-0.5" />
          <span>
            <strong className="text-primary">Pro tip:</strong> Use categories to quickly allow
            groups of related apps. For example, select <strong>@coding</strong> to allow all
            development tools at once.
          </span>
        </p>
      </div>

      {/* Predefined Categories - from backend */}
      <div role="group" aria-labelledby="categories-label">
        <p id="categories-label" className="text-sm font-medium mb-2 flex items-center gap-2">
          Categories
          <Tooltip>
            <TooltipTrigger asChild>
              <HelpCircle className="h-4 w-4 text-muted-foreground cursor-help" />
            </TooltipTrigger>
            <TooltipContent className="max-w-xs">
              Categories are groups of related apps. Click a category to quickly allow all apps in
              that group. Selected categories are highlighted.
            </TooltipContent>
          </Tooltip>
        </p>
        <div className="flex flex-wrap gap-2">
          {categories?.map((category) => {
            const categoryDescription = `${category.description}. Includes: ${category.exampleApps.slice(0, 3).join(", ")}${category.exampleApps.length > 3 ? `, and more` : ""}`;
            const selected = isCategorySelected(category);
            return (
              <Tooltip key={category.id}>
                <TooltipTrigger asChild>
                  <Badge
                    variant={selected ? "default" : "outline"}
                    className="cursor-pointer hover:bg-primary/80 transition-colors"
                    onClick={() => onToggleCategory(category.id)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        onToggleCategory(category.id);
                      }
                    }}
                    role="checkbox"
                    aria-checked={selected}
                    aria-label={`${category.name} category${selected ? " (selected)" : ""}`}
                    tabIndex={0}
                  >
                    {CATEGORY_ICONS[category.id] ?? <FileText className="h-4 w-4" />}
                    <span className="ml-1">{category.id}</span>
                    {selected && <Check className="h-3 w-3 ml-1" aria-hidden="true" />}
                  </Badge>
                </TooltipTrigger>
                <TooltipContent className="max-w-xs">{categoryDescription}</TooltipContent>
              </Tooltip>
            );
          })}
        </div>
      </div>

      {/* Individual Apps - from backend */}
      <div role="group" aria-labelledby="apps-label">
        <p id="apps-label" className="text-sm font-medium mb-2">
          Applications
        </p>
        <ScrollArea className="h-[300px] border rounded-md p-2">
          <div className="space-y-1" role="listbox" aria-label="Available applications">
            {filteredApps.length > 0 ? (
              filteredApps.map((app) => {
                const isSelected = selectedApps.includes(app);
                return (
                  <div
                    key={app}
                    className={`flex items-center justify-between p-2 rounded-md cursor-pointer transition-colors ${
                      isSelected ? "bg-primary text-primary-foreground" : "hover:bg-secondary"
                    }`}
                    onClick={() => onToggleApp(app)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        onToggleApp(app);
                      }
                    }}
                    role="option"
                    aria-selected={isSelected}
                    tabIndex={0}
                  >
                    <span className="text-sm">{app}</span>
                    {isSelected && <Check className="h-4 w-4" aria-hidden="true" />}
                  </div>
                );
              })
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <p className="text-sm">No apps found</p>
                {searchQuery && <p className="text-xs mt-1">Try a different search term</p>}
              </div>
            )}
          </div>
        </ScrollArea>
      </div>

      {/* Selected Count */}
      <div className="flex items-center justify-between pt-2 border-t">
        <p className="text-sm text-muted-foreground flex items-center gap-1">
          {selectedApps.length} app{selectedApps.length !== 1 ? "s" : ""} selected
          <Tooltip>
            <TooltipTrigger asChild>
              <HelpCircle className="h-3 w-3 cursor-help" />
            </TooltipTrigger>
            <TooltipContent>
              These apps will be accessible during Focus Time. All other apps will be blocked.
            </TooltipContent>
          </Tooltip>
        </p>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button onClick={onClose} size="sm">
              Done
            </Button>
          </TooltipTrigger>
          <TooltipContent>Save changes and close</TooltipContent>
        </Tooltip>
      </div>
    </div>
  );
}
