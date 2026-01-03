import {
  Timer,
  LayoutDashboard,
  Calendar as CalendarIcon,
  Flame,
  BarChart3,
  Trophy,
  BookOpen,
  Bot,
  Users,
  Shield,
} from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarHeader,
  SidebarFooter,
  SidebarRail,
  useSidebar,
} from "@/components/ui/sidebar";

export type ViewType =
  | "timer"
  | "dashboard"
  | "calendar"
  | "streaks"
  | "analytics"
  | "achievements"
  | "journal"
  | "coach"
  | "team"
  | "blocking";

interface AppSidebarProps {
  activeView: ViewType;
  onViewChange: (view: ViewType) => void;
}

const navigationItems: Array<{
  id: ViewType;
  label: string;
  icon: typeof Timer;
  group: "focus" | "progress" | "tools" | "settings";
}> = [
  { id: "timer", label: "Focus Timer", icon: Timer, group: "focus" },
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard, group: "focus" },
  { id: "calendar", label: "Calendar", icon: CalendarIcon, group: "focus" },
  { id: "streaks", label: "Streaks", icon: Flame, group: "progress" },
  { id: "analytics", label: "Analytics", icon: BarChart3, group: "progress" },
  { id: "achievements", label: "Achievements", icon: Trophy, group: "progress" },
  { id: "journal", label: "Journal", icon: BookOpen, group: "tools" },
  { id: "coach", label: "AI Coach", icon: Bot, group: "tools" },
  { id: "team", label: "Team", icon: Users, group: "tools" },
  { id: "blocking", label: "Blocking", icon: Shield, group: "settings" },
];

export function AppSidebar({ activeView, onViewChange }: AppSidebarProps) {
  const { isMobile, setOpenMobile } = useSidebar();

  const handleNavigation = (view: ViewType) => {
    onViewChange(view);
    // Auto-close mobile drawer after selection
    if (isMobile) {
      setOpenMobile(false);
    }
  };

  const focusItems = navigationItems.filter((item) => item.group === "focus");
  const progressItems = navigationItems.filter((item) => item.group === "progress");
  const toolsItems = navigationItems.filter((item) => item.group === "tools");
  const settingsItems = navigationItems.filter((item) => item.group === "settings");

  return (
    <Sidebar collapsible="icon">
      {/* Header */}
      <SidebarHeader className="border-b">
        <div className="flex items-center gap-2 px-2 py-1">
          <Timer className="h-6 w-6 shrink-0" />
          <span className="text-lg font-semibold group-data-[collapsible=icon]:hidden">
            FocusFlow
          </span>
        </div>
      </SidebarHeader>

      {/* Navigation Content */}
      <SidebarContent>
        {/* Focus Section */}
        <SidebarGroup>
          <SidebarGroupLabel>Focus</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {focusItems.map((item) => (
                <SidebarMenuItem key={item.id}>
                  <SidebarMenuButton
                    onClick={() => handleNavigation(item.id)}
                    isActive={activeView === item.id}
                    tooltip={item.label}
                  >
                    <item.icon className="h-4 w-4" aria-hidden="true" />
                    <span>{item.label}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Progress Section */}
        <SidebarGroup>
          <SidebarGroupLabel>Progress</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {progressItems.map((item) => (
                <SidebarMenuItem key={item.id}>
                  <SidebarMenuButton
                    onClick={() => handleNavigation(item.id)}
                    isActive={activeView === item.id}
                    tooltip={item.label}
                  >
                    <item.icon className="h-4 w-4" aria-hidden="true" />
                    <span>{item.label}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Tools Section */}
        <SidebarGroup>
          <SidebarGroupLabel>Tools</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {toolsItems.map((item) => (
                <SidebarMenuItem key={item.id}>
                  <SidebarMenuButton
                    onClick={() => handleNavigation(item.id)}
                    isActive={activeView === item.id}
                    tooltip={item.label}
                  >
                    <item.icon className="h-4 w-4" aria-hidden="true" />
                    <span>{item.label}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {/* Settings Section */}
        <SidebarGroup>
          <SidebarGroupLabel>Settings</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {settingsItems.map((item) => (
                <SidebarMenuItem key={item.id}>
                  <SidebarMenuButton
                    onClick={() => handleNavigation(item.id)}
                    isActive={activeView === item.id}
                    tooltip={item.label}
                  >
                    <item.icon className="h-4 w-4" aria-hidden="true" />
                    <span>{item.label}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      {/* Footer */}
      <SidebarFooter className="border-t">
        <div className="px-3 py-2 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
          v0.1.0
        </div>
      </SidebarFooter>

      {/* Rail for resize/collapse */}
      <SidebarRail />
    </Sidebar>
  );
}
