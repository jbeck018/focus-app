// features/FocusTime.tsx - Focus Time calendar-based blocking feature

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import {
  useFocusTimeEvents,
  useFocusTimeState,
  useFocusTimeActions,
  useRefreshFocusTimeEvents,
} from "@/hooks/useFocusTime";
import { useCalendarConnections } from "@/hooks/useCalendar";
import { formatTime } from "@/hooks/useTimer";
import { FOCUS_TIME_INSTRUCTIONS, FOCUS_TIME_CATEGORIES } from "@focusflow/types";
import {
  Target,
  Calendar as CalendarIcon,
  Clock,
  Play,
  Loader2,
  AlertCircle,
  Info,
  RefreshCw,
  CheckCircle,
} from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

export function FocusTime() {
  const { data: connections } = useCalendarConnections();
  const { data: events, isLoading: eventsLoading } = useFocusTimeEvents();
  const { data: state } = useFocusTimeState();
  const { startNow } = useFocusTimeActions();
  const refresh = useRefreshFocusTimeEvents();
  const [expandedEvent, setExpandedEvent] = useState<string | null>(null);

  const hasConnection = connections?.some((c) => c.connected);
  const connectedProvider = connections?.find((c) => c.connected)?.provider;

  if (!hasConnection) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <Target className="h-5 w-5 text-green-500" />
            Focus Time
          </h2>
          <p className="text-sm text-muted-foreground">
            Automatic blocking based on your calendar events
          </p>
        </div>

        <Card>
          <CardContent className="py-8 text-center">
            <CalendarIcon className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <p className="text-muted-foreground">No calendar connected</p>
            <p className="text-sm text-muted-foreground mt-1">
              Connect your calendar to automatically block distractions during Focus Time
            </p>
            <Button
              variant="outline"
              className="mt-4"
              onClick={() => (window.location.hash = "#calendar-connections")}
            >
              Connect Calendar
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <Target className="h-5 w-5 text-green-500" />
            Focus Time
          </h2>
          <p className="text-sm text-muted-foreground">
            Automatic blocking based on your calendar events
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => refresh.mutate()}
          disabled={refresh.isPending}
        >
          {refresh.isPending ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="h-4 w-4" />
          )}
          <span className="ml-2">Refresh</span>
        </Button>
      </div>

      {/* Setup Instructions */}
      {connectedProvider && (
        <Alert>
          <Info className="h-4 w-4" />
          <AlertTitle>{FOCUS_TIME_INSTRUCTIONS[connectedProvider].title}</AlertTitle>
          <AlertDescription>
            <ol className="list-decimal list-inside space-y-1 mt-2">
              {FOCUS_TIME_INSTRUCTIONS[connectedProvider].steps.map((step, idx) => (
                <li key={idx} className="text-sm">
                  {step}
                </li>
              ))}
            </ol>
            <div className="mt-3 p-2 bg-secondary rounded-md">
              <p className="text-xs font-medium mb-1">Example:</p>
              <pre className="text-xs text-muted-foreground whitespace-pre-line">
                {FOCUS_TIME_INSTRUCTIONS[connectedProvider].example}
              </pre>
            </div>
          </AlertDescription>
        </Alert>
      )}

      {/* Active Focus Time */}
      {state?.active && state.current_event && (
        <Card className="border-green-500/50 bg-green-500/10">
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <CheckCircle className="h-4 w-4 text-green-500" />
              Currently Active
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">{state.current_event.title}</p>
                <div className="flex items-center gap-2 mt-1">
                  <Clock className="h-3 w-3 text-muted-foreground" />
                  <p className="text-xs text-muted-foreground">
                    {formatTime(state.remaining_seconds)} remaining
                  </p>
                </div>
              </div>
              <Badge
                variant="secondary"
                className="text-green-700 bg-green-100 dark:bg-green-900 dark:text-green-300"
              >
                Active
              </Badge>
            </div>
            <div className="mt-3 pt-3 border-t border-border/50">
              <p className="text-xs text-muted-foreground mb-1">Allowed apps:</p>
              <div className="flex flex-wrap gap-1">
                {state.allowed_apps.map((app) => (
                  <Badge key={app} variant="outline" className="text-xs">
                    {app}
                  </Badge>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Upcoming Focus Time Events */}
      <Card>
        <CardHeader>
          <CardTitle className="text-sm font-medium">Upcoming Focus Time</CardTitle>
        </CardHeader>
        <CardContent>
          {eventsLoading ? (
            <div className="flex items-center justify-center p-8">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : events && events.length > 0 ? (
            <Accordion
              type="single"
              collapsible
              value={expandedEvent || ""}
              onValueChange={setExpandedEvent}
            >
              {events.map((event) => (
                <AccordionItem key={event.id} value={event.id}>
                  <AccordionTrigger className="hover:no-underline">
                    <div className="flex items-center justify-between flex-1 pr-2">
                      <div className="flex items-start gap-3 text-left">
                        <Target className="h-4 w-4 text-green-500 mt-1 flex-shrink-0" />
                        <div>
                          <p className="font-medium">{event.title}</p>
                          <p className="text-xs text-muted-foreground mt-0.5">
                            {formatEventTime(event.start_time)} - {formatEventTime(event.end_time)}
                          </p>
                        </div>
                      </div>
                      {event.is_active && (
                        <Badge
                          variant="secondary"
                          className="text-green-700 bg-green-100 dark:bg-green-900 dark:text-green-300"
                        >
                          Active
                        </Badge>
                      )}
                    </div>
                  </AccordionTrigger>
                  <AccordionContent>
                    <div className="pl-7 space-y-3">
                      {/* Allowed Apps */}
                      <div>
                        <p className="text-xs font-medium text-muted-foreground mb-1">
                          Allowed Apps:
                        </p>
                        <div className="flex flex-wrap gap-1">
                          {event.allowed_apps.length > 0 ? (
                            event.allowed_apps.map((app) => (
                              <Badge key={app} variant="outline" className="text-xs">
                                {app}
                              </Badge>
                            ))
                          ) : (
                            <p className="text-xs text-muted-foreground">All apps blocked</p>
                          )}
                        </div>
                      </div>

                      {/* Categories */}
                      {event.allowed_categories.length > 0 && (
                        <div>
                          <p className="text-xs font-medium text-muted-foreground mb-1">
                            Categories:
                          </p>
                          <div className="flex flex-wrap gap-1">
                            {event.allowed_categories.map((category) => (
                              <Badge key={category} variant="secondary" className="text-xs">
                                {category}
                              </Badge>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Actions */}
                      {!event.is_active && (
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => startNow.mutate(event.id)}
                          disabled={startNow.isPending}
                        >
                          <Play className="h-3 w-3 mr-1" />
                          Start Now
                        </Button>
                      )}
                    </div>
                  </AccordionContent>
                </AccordionItem>
              ))}
            </Accordion>
          ) : (
            <div className="text-center py-8">
              <AlertCircle className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
              <p className="text-muted-foreground">No Focus Time events found</p>
              <p className="text-sm text-muted-foreground mt-1">
                Create a calendar event with title starting with "🎯 Focus Time"
              </p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Available Categories Reference */}
      <Card className="bg-muted/50">
        <CardHeader>
          <CardTitle className="text-sm font-medium">Available Categories</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-2 md:grid-cols-2">
            {(Object.entries(FOCUS_TIME_CATEGORIES) as [string, readonly string[]][]).map(
              ([category, apps]) => (
                <div key={category} className="text-xs">
                  <Badge variant="outline" className="mb-1">
                    {category}
                  </Badge>
                  <p className="text-muted-foreground ml-2">{apps.join(", ")}</p>
                </div>
              )
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function formatEventTime(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });
}
