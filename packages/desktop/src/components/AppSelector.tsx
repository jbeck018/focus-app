// components/AppSelector.tsx - App selection component for Focus Time overrides

import { useState, useMemo } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { FOCUS_TIME_CATEGORIES, type FocusTimeCategory } from "@focusflow/types";
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
} from "lucide-react";

interface AppSelectorProps {
  selectedApps: string[];
  onToggleApp: (appName: string) => void;
  onToggleCategory: (category: FocusTimeCategory) => void;
  onClose: () => void;
}

const CATEGORY_ICONS: Record<FocusTimeCategory, React.ReactNode> = {
  "@coding": <Laptop className="h-4 w-4" />,
  "@communication": <MessageSquare className="h-4 w-4" />,
  "@browser": <Globe className="h-4 w-4" />,
  "@design": <Palette className="h-4 w-4" />,
  "@productivity": <FileText className="h-4 w-4" />,
  "@terminal": <Terminal className="h-4 w-4" />,
  "@music": <Music className="h-4 w-4" />,
};

export function AppSelector({
  selectedApps,
  onToggleApp,
  onToggleCategory,
  onClose,
}: AppSelectorProps) {
  const [searchQuery, setSearchQuery] = useState("");

  // Flatten all apps from categories
  const allApps = useMemo(() => {
    const apps = new Set<string>();
    Object.values(FOCUS_TIME_CATEGORIES).forEach((categoryApps) => {
      categoryApps.forEach((app) => apps.add(app));
    });
    return Array.from(apps).sort();
  }, []);

  // Filter apps based on search
  const filteredApps = useMemo(() => {
    if (!searchQuery) return allApps;
    const query = searchQuery.toLowerCase();
    return allApps.filter((app) => app.toLowerCase().includes(query));
  }, [allApps, searchQuery]);

  // Check if category is fully selected
  const isCategorySelected = (category: FocusTimeCategory) => {
    const categoryApps = FOCUS_TIME_CATEGORIES[category];
    return categoryApps.every((app) => selectedApps.includes(app));
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <Search className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
        <Input
          placeholder="Search apps..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="flex-1"
          aria-label="Search applications"
        />
      </div>

      {/* Predefined Categories */}
      <div role="group" aria-labelledby="categories-label">
        <p id="categories-label" className="text-sm font-medium mb-2">
          Categories
        </p>
        <div className="flex flex-wrap gap-2">
          {(Object.keys(FOCUS_TIME_CATEGORIES) as FocusTimeCategory[]).map((category) => (
            <Badge
              key={category}
              variant={isCategorySelected(category) ? "default" : "outline"}
              className="cursor-pointer hover:bg-primary/80 transition-colors"
              onClick={() => onToggleCategory(category)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  onToggleCategory(category);
                }
              }}
              role="checkbox"
              aria-checked={isCategorySelected(category)}
              aria-label={`${category} category${isCategorySelected(category) ? " (selected)" : ""}`}
              tabIndex={0}
            >
              {CATEGORY_ICONS[category]}
              <span className="ml-1">{category}</span>
              {isCategorySelected(category) && (
                <Check className="h-3 w-3 ml-1" aria-hidden="true" />
              )}
            </Badge>
          ))}
        </div>
      </div>

      {/* Individual Apps */}
      <div role="group" aria-labelledby="apps-label">
        <p id="apps-label" className="text-sm font-medium mb-2">
          Applications
        </p>
        <ScrollArea className="h-[300px] border rounded-md p-2">
          <div className="space-y-1" role="listbox" aria-label="Available applications">
            {filteredApps.map((app) => {
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
            })}
          </div>
        </ScrollArea>
      </div>

      {/* Selected Count */}
      <div className="flex items-center justify-between pt-2 border-t">
        <p className="text-sm text-muted-foreground">
          {selectedApps.length} app{selectedApps.length !== 1 ? "s" : ""} selected
        </p>
        <Button onClick={onClose} size="sm">
          Done
        </Button>
      </div>
    </div>
  );
}
