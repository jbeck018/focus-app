// features/Calendar.tsx - Calendar integration and schedule view

import { useState, useMemo } from "react";
import { open } from "@tauri-apps/plugin-shell";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  useCalendarConnections,
  useCalendarEvents,
  useFocusSuggestions,
  useMeetingLoad,
  useStartCalendarOAuth,
  useDisconnectCalendar,
  useOAuthConfigStatus,
} from "@/hooks/useCalendar";
import type { CalendarProvider, CalendarEvent } from "@focusflow/types";
import { PROVIDER_INFO } from "@focusflow/types";
import {
  Calendar as CalendarIcon,
  MapPin,
  Link2,
  Link2Off,
  Loader2,
  Lightbulb,
  BarChart3,
  ExternalLink,
  AlertTriangle,
  Info,
  Target,
} from "lucide-react";
import { useFocusTimeEvents } from "@/hooks/useFocusTime";
import { FOCUS_TIME_INSTRUCTIONS } from "@focusflow/types";

export function Calendar() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <CalendarIcon className="h-5 w-5" />
          Calendar
        </h2>
        <p className="text-sm text-muted-foreground">
          Connect your calendar to find optimal focus times
        </p>
      </div>

      <Tabs defaultValue="schedule">
        <TabsList>
          <TabsTrigger value="schedule">Schedule</TabsTrigger>
          <TabsTrigger value="focus-time">Focus Time</TabsTrigger>
          <TabsTrigger value="insights">Insights</TabsTrigger>
          <TabsTrigger value="connections">Connections</TabsTrigger>
        </TabsList>

        <TabsContent value="schedule" className="mt-4">
          <ScheduleView />
        </TabsContent>

        <TabsContent value="focus-time" className="mt-4">
          <FocusTimeTabView />
        </TabsContent>

        <TabsContent value="insights" className="mt-4">
          <InsightsView />
        </TabsContent>

        <TabsContent value="connections" className="mt-4">
          <ConnectionsView />
        </TabsContent>
      </Tabs>
    </div>
  );
}

function ScheduleView() {
  // Calculate today and tomorrow dates in a stable way (memoized to avoid recalculation on every render)
  const { today, tomorrow } = useMemo(() => {
    const now = new Date();
    const todayDate = now.toISOString().split("T")[0];
    const tomorrowDate = new Date(now.getTime() + 86400000).toISOString().split("T")[0];
    return { today: todayDate, tomorrow: tomorrowDate };
  }, []);

  const { data: events, isLoading } = useCalendarEvents(today, tomorrow);
  const { data: suggestions } = useFocusSuggestions();
  const { data: connections } = useCalendarConnections();

  const hasConnection = connections?.some((c) => c.connected);

  if (!hasConnection) {
    return (
      <Card>
        <CardContent className="py-8 text-center">
          <CalendarIcon className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground">No calendar connected</p>
          <p className="text-sm text-muted-foreground mt-1">
            Connect your calendar to see your schedule and find focus time
          </p>
          <Button
            variant="outline"
            className="mt-4"
            onClick={() => (window.location.hash = "#connections")}
          >
            Connect Calendar
          </Button>
        </CardContent>
      </Card>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Focus Suggestions */}
      {suggestions && suggestions.length > 0 && (
        <Card className="border-primary/20 bg-primary/5">
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Lightbulb className="h-4 w-4 text-primary" />
              Suggested Focus Blocks
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {suggestions.map((suggestion, idx) => (
                <div
                  key={idx}
                  className="flex items-center justify-between p-3 bg-background rounded-lg"
                >
                  <div>
                    <p className="text-sm font-medium">
                      {formatTime(suggestion.start_time)} - {formatTime(suggestion.end_time)}
                    </p>
                    <p className="text-xs text-muted-foreground">{suggestion.reason}</p>
                  </div>
                  <Badge variant="secondary">{suggestion.duration_minutes} min</Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Today's Events */}
      <Card>
        <CardHeader>
          <CardTitle className="text-sm font-medium">Today's Schedule</CardTitle>
        </CardHeader>
        <CardContent>
          {events && events.length > 0 ? (
            <div className="space-y-3">
              {events.map((event) => (
                <EventCard key={event.id} event={event} />
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground text-center py-4">
              No events scheduled for today
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function EventCard({ event }: { event: CalendarEvent }) {
  const providerInfo = PROVIDER_INFO[event.provider];

  return (
    <div className="flex items-start gap-3 p-3 rounded-lg bg-secondary/30">
      <div className="flex-shrink-0 w-16 text-center">
        <p className="text-sm font-medium">{formatTime(event.start_time)}</p>
        <p className="text-xs text-muted-foreground">{formatTime(event.end_time)}</p>
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <p className="font-medium truncate">{event.title}</p>
          {event.is_busy && (
            <Badge variant="secondary" className="text-xs">
              Busy
            </Badge>
          )}
        </div>
        {event.location && (
          <p className="text-xs text-muted-foreground flex items-center gap-1 mt-1">
            <MapPin className="h-3 w-3" />
            {event.location}
          </p>
        )}
      </div>
      <span className="text-sm" title={providerInfo.label}>
        {providerInfo.icon}
      </span>
    </div>
  );
}

function InsightsView() {
  const { data: meetingLoad, isLoading } = useMeetingLoad();
  const { data: connections } = useCalendarConnections();

  const hasConnection = connections?.some((c) => c.connected);

  if (!hasConnection) {
    return (
      <Card>
        <CardContent className="py-8 text-center">
          <BarChart3 className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground">Connect a calendar to see insights</p>
        </CardContent>
      </Card>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">Meeting Load This Week</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-3xl font-bold">
            {meetingLoad?.total_meeting_hours_this_week.toFixed(1)}h
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            ~{meetingLoad?.average_daily_meetings.toFixed(1)} meetings per day
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">Longest Free Block</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-3xl font-bold">{meetingLoad?.longest_free_block_minutes} min</div>
          <p className="text-xs text-muted-foreground mt-1">Great for deep work</p>
        </CardContent>
      </Card>

      {meetingLoad?.busiest_day && (
        <Card className="md:col-span-2">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Busiest Day</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-medium">{meetingLoad.busiest_day}</p>
            <p className="text-xs text-muted-foreground mt-1">
              Consider protecting focus time on this day
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function ConnectionsView() {
  const { data: connections, isLoading } = useCalendarConnections();
  const { data: oauthConfig } = useOAuthConfigStatus();
  const startOAuth = useStartCalendarOAuth();
  const disconnect = useDisconnectCalendar();
  const [connecting, setConnecting] = useState<CalendarProvider | null>(null);
  const [oauthError, setOauthError] = useState<string | null>(null);

  // Check if a provider's OAuth is configured
  const isProviderConfigured = (provider: CalendarProvider): boolean => {
    if (!oauthConfig) return true; // Assume configured if status not loaded
    return provider === "google" ? oauthConfig.google_configured : oauthConfig.microsoft_configured;
  };

  // Get setup URL for a provider
  const getSetupUrl = (provider: CalendarProvider): string => {
    if (!oauthConfig) return "";
    return provider === "google" ? oauthConfig.google_setup_url : oauthConfig.microsoft_setup_url;
  };

  const handleConnect = async (provider: CalendarProvider) => {
    setConnecting(provider);
    setOauthError(null);
    try {
      const result = await startOAuth.mutateAsync(provider);
      // Open the OAuth URL in the default browser
      await open(result.url);
      // Note: The actual OAuth callback handling would be done via deep linking
    } catch (error) {
      // Check if it's an OAuth not configured error
      const errorMessage = error instanceof Error ? error.message : String(error);
      if (errorMessage.includes("not configured") || errorMessage.includes("OAuthNotConfigured")) {
        setOauthError(errorMessage);
      } else {
        console.error("Failed to start OAuth:", error);
        setOauthError(`Failed to connect: ${errorMessage}`);
      }
    } finally {
      setConnecting(null);
    }
  };

  const handleDisconnect = async (provider: CalendarProvider) => {
    await disconnect.mutateAsync(provider);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        Connect your calendars to see your schedule and find optimal focus times.
      </p>

      {/* OAuth Error Alert */}
      {oauthError && (
        <Card className="border-amber-500/50 bg-amber-500/10">
          <CardContent className="py-4">
            <div className="flex items-start gap-3">
              <AlertTriangle className="h-5 w-5 text-amber-500 flex-shrink-0 mt-0.5" />
              <div className="flex-1">
                <p className="text-sm font-medium text-amber-700 dark:text-amber-400">
                  Calendar Integration Not Available
                </p>
                <p className="text-xs text-muted-foreground mt-1 whitespace-pre-line">
                  {oauthError}
                </p>
                <Button
                  variant="ghost"
                  size="sm"
                  className="mt-2 h-7 px-2 text-xs"
                  onClick={() => setOauthError(null)}
                >
                  Dismiss
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Filter to only show Google Calendar for now - Microsoft/Azure OAuth is not configured.
          To re-enable Microsoft, remove the .filter() call below */}
      <div className="grid gap-4 md:grid-cols-2">
        {connections
          ?.filter((c) => c.provider !== "microsoft")
          .map((connection) => {
            const info = PROVIDER_INFO[connection.provider];
            const isConnecting = connecting === connection.provider;
            const isDisconnecting = disconnect.isPending;
            const isConfigured = isProviderConfigured(connection.provider);

            return (
              <Card key={connection.provider} className={!isConfigured ? "opacity-75" : ""}>
                <CardContent className="pt-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <span className="text-2xl">{info.icon}</span>
                      <div>
                        <p className="font-medium">{info.label}</p>
                        {connection.connected && connection.email && (
                          <p className="text-xs text-muted-foreground">{connection.email}</p>
                        )}
                        {!isConfigured && !connection.connected && (
                          <p className="text-xs text-amber-600 dark:text-amber-400 flex items-center gap-1">
                            <Info className="h-3 w-3" />
                            Setup required
                          </p>
                        )}
                      </div>
                    </div>

                    {connection.connected ? (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleDisconnect(connection.provider)}
                        disabled={isDisconnecting}
                      >
                        {isDisconnecting ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <>
                            <Link2Off className="h-4 w-4 mr-1" />
                            Disconnect
                          </>
                        )}
                      </Button>
                    ) : (
                      <Button
                        size="sm"
                        onClick={() => handleConnect(connection.provider)}
                        disabled={isConnecting}
                        variant={isConfigured ? "default" : "outline"}
                      >
                        {isConnecting ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <>
                            <Link2 className="h-4 w-4 mr-1" />
                            Connect
                          </>
                        )}
                      </Button>
                    )}
                  </div>

                  {connection.connected && connection.last_sync && (
                    <p className="text-xs text-muted-foreground mt-3">
                      Last synced: {formatRelativeTime(connection.last_sync)}
                    </p>
                  )}

                  {/* Setup instructions for unconfigured providers */}
                  {!isConfigured && !connection.connected && (
                    <div className="mt-3 pt-3 border-t border-border/50">
                      <p className="text-xs text-muted-foreground">
                        OAuth credentials need to be configured.{" "}
                        <button
                          className="text-primary hover:underline"
                          onClick={() => open(getSetupUrl(connection.provider))}
                        >
                          View setup guide
                        </button>
                      </p>
                    </div>
                  )}
                </CardContent>
              </Card>
            );
          })}
      </div>

      <Card className="bg-muted/50">
        <CardContent className="py-4">
          <div className="flex items-start gap-3">
            <ExternalLink className="h-5 w-5 text-muted-foreground flex-shrink-0 mt-0.5" />
            <div>
              <p className="text-sm font-medium">Privacy First</p>
              <p className="text-xs text-muted-foreground mt-1">
                Calendar data is fetched on-demand and processed locally. We only store connection
                tokens securely on your device. No calendar data is ever sent to our servers.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function formatTime(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });
}

function formatRelativeTime(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);

  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;

  return date.toLocaleDateString();
}

function FocusTimeTabView() {
  const { data: focusTimeEvents, isLoading } = useFocusTimeEvents();
  const { data: connections } = useCalendarConnections();

  const hasConnection = connections?.some((c) => c.connected);
  const connectedProvider = connections?.find((c) => c.connected)?.provider;

  if (!hasConnection) {
    return (
      <Card>
        <CardContent className="py-8 text-center">
          <Target className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground">Connect a calendar to use Focus Time</p>
          <p className="text-sm text-muted-foreground mt-1">
            Automatically block distractions during scheduled Focus Time events
          </p>
        </CardContent>
      </Card>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Setup Instructions */}
      {connectedProvider && (
        <Card className="bg-primary/5 border-primary/20">
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Info className="h-4 w-4 text-primary" />
              {FOCUS_TIME_INSTRUCTIONS[connectedProvider].title}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ol className="list-decimal list-inside space-y-1 text-sm">
              {FOCUS_TIME_INSTRUCTIONS[connectedProvider].steps.map((step, idx) => (
                <li key={idx}>{step}</li>
              ))}
            </ol>
            <div className="mt-3 p-3 bg-background rounded-md">
              <p className="text-xs font-medium mb-1">Example:</p>
              <pre className="text-xs text-muted-foreground whitespace-pre-line">
                {FOCUS_TIME_INSTRUCTIONS[connectedProvider].example}
              </pre>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Focus Time Events */}
      <Card>
        <CardHeader>
          <CardTitle className="text-sm font-medium">Focus Time Events</CardTitle>
        </CardHeader>
        <CardContent>
          {focusTimeEvents && focusTimeEvents.length > 0 ? (
            <div className="space-y-3">
              {focusTimeEvents.map((event) => (
                <div
                  key={event.id}
                  className="flex items-start gap-3 p-3 rounded-lg bg-secondary/30"
                >
                  <Target className="h-5 w-5 text-green-500 mt-0.5 flex-shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="font-medium truncate">{event.title}</p>
                      {event.is_active && (
                        <Badge variant="secondary" className="text-xs">
                          Active
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">
                      {formatTime(event.start_time)} - {formatTime(event.end_time)}
                    </p>
                    {event.allowed_apps.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {event.allowed_apps.slice(0, 3).map((app) => (
                          <Badge key={app} variant="outline" className="text-xs">
                            {app}
                          </Badge>
                        ))}
                        {event.allowed_apps.length > 3 && (
                          <Badge variant="outline" className="text-xs">
                            +{event.allowed_apps.length - 3} more
                          </Badge>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground text-center py-4">
              No Focus Time events found. Create a calendar event with title starting with "🎯 Focus
              Time"
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
