// features/FocusTimeHelp.tsx - Help and Getting Started page for Focus Time

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
import { FOCUS_TIME_CATEGORIES } from "@focusflow/types";
import { navigateTo } from "@/hooks/useNavigation";
import {
  Target,
  Calendar,
  Clock,
  Shield,
  Lightbulb,
  Copy,
  CheckCircle,
  ArrowLeft,
  BookOpen,
  Zap,
  AlertCircle,
} from "lucide-react";

interface ExampleEvent {
  title: string;
  time: string;
  description: string;
  useCase: string;
  apps: string[];
  categories: string[];
}

const EXAMPLE_EVENTS: ExampleEvent[] = [
  {
    title: "🎯 Focus Time: Deep Work Session",
    time: "9:00 AM - 11:00 AM",
    description: "@coding @terminal @music",
    useCase: "Perfect for coding sessions with background music",
    apps: ["Visual Studio Code", "Terminal", "Spotify"],
    categories: ["@coding", "@terminal", "@music"],
  },
  {
    title: "🎯 Focus Time: Meeting Prep",
    time: "2:00 PM - 2:30 PM",
    description: "@productivity @browser Notion",
    useCase: "Prepare presentations or documentation before meetings",
    apps: ["Notion", "Google Chrome"],
    categories: ["@productivity", "@browser"],
  },
  {
    title: "🎯 Focus Time: UI Design",
    time: "3:00 PM - 5:00 PM",
    description: "@design @browser",
    useCase: "Design work with reference browsing",
    apps: ["Figma", "Sketch", "Google Chrome"],
    categories: ["@design", "@browser"],
  },
  {
    title: "🎯 Focus Time: Writing",
    time: "10:00 AM - 12:00 PM",
    description: "@productivity @music Obsidian",
    useCase: "Distraction-free writing with specific tools",
    apps: ["Obsidian", "Spotify"],
    categories: ["@productivity", "@music"],
  },
];

export function FocusTimeHelp() {
  const [copiedText, setCopiedText] = useState<string | null>(null);

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    setCopiedText(label);
    setTimeout(() => setCopiedText(null), 2000);
  };

  return (
    <div className="space-y-6 max-w-4xl">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold flex items-center gap-2">
            <BookOpen className="h-6 w-6 text-green-500" />
            Focus Time Help & Getting Started
          </h2>
          <p className="text-sm text-muted-foreground mt-1">
            Everything you need to know about calendar-based blocking
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => navigateTo("focus-time")}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back to Focus Time
        </Button>
      </div>

      {/* What is Focus Time */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Target className="h-5 w-5 text-green-500" />
            What is Focus Time?
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground">
            Focus Time is a calendar-based blocking system that automatically blocks distracting
            apps during scheduled focus sessions. Unlike regular blocking that requires manual
            activation, Focus Time:
          </p>
          <ul className="space-y-2 ml-4">
            <li className="text-sm flex items-start gap-2">
              <Calendar className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
              <span>
                <strong>Syncs with your calendar</strong> - Works with Google Calendar and Microsoft
                Outlook
              </span>
            </li>
            <li className="text-sm flex items-start gap-2">
              <Clock className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
              <span>
                <strong>Activates automatically</strong> - No need to manually start/stop blocking
              </span>
            </li>
            <li className="text-sm flex items-start gap-2">
              <Shield className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
              <span>
                <strong>Whitelist approach</strong> - Specify which apps you need, everything else
                is blocked
              </span>
            </li>
            <li className="text-sm flex items-start gap-2">
              <Zap className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
              <span>
                <strong>Smart categories</strong> - Use predefined categories like @coding, @design,
                or @communication
              </span>
            </li>
          </ul>
        </CardContent>
      </Card>

      {/* How to Create Focus Time Events */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5 text-green-500" />
            How to Create a Focus Time Event
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Accordion type="single" collapsible>
            <AccordionItem value="google">
              <AccordionTrigger>Google Calendar</AccordionTrigger>
              <AccordionContent className="space-y-3">
                <ol className="list-decimal list-inside space-y-2 text-sm">
                  <li>Open Google Calendar</li>
                  <li>
                    Create a new event with title starting with one of these:
                    <div className="flex flex-wrap gap-1 mt-2">
                      <Badge variant="outline" className="font-mono">
                        🎯 Focus Time:
                      </Badge>
                      <Badge variant="outline" className="font-mono">
                        [Focus]
                      </Badge>
                      <Badge variant="outline" className="font-mono">
                        Focus:
                      </Badge>
                    </div>
                  </li>
                  <li>Set your desired start and end time</li>
                  <li>
                    In the description field, add allowed apps using categories or specific app
                    names
                  </li>
                  <li>Save the event</li>
                </ol>
                <div className="mt-4 p-3 bg-muted rounded-lg">
                  <p className="text-xs font-medium mb-2">Example:</p>
                  <div className="space-y-1">
                    <p className="text-sm font-mono">🎯 Focus Time: Deep Work</p>
                    <p className="text-xs text-muted-foreground">9:00 AM - 11:00 AM</p>
                    <p className="text-xs text-muted-foreground font-mono mt-2">
                      Description: @coding @terminal @music
                    </p>
                  </div>
                </div>
              </AccordionContent>
            </AccordionItem>

            <AccordionItem value="outlook">
              <AccordionTrigger>Microsoft Outlook</AccordionTrigger>
              <AccordionContent className="space-y-3">
                <ol className="list-decimal list-inside space-y-2 text-sm">
                  <li>Open Outlook Calendar</li>
                  <li>
                    Create a new event with title starting with one of these:
                    <div className="flex flex-wrap gap-1 mt-2">
                      <Badge variant="outline" className="font-mono">
                        🎯 Focus Time:
                      </Badge>
                      <Badge variant="outline" className="font-mono">
                        [Focus]
                      </Badge>
                      <Badge variant="outline" className="font-mono">
                        Focus:
                      </Badge>
                    </div>
                  </li>
                  <li>Set your desired start and end time</li>
                  <li>
                    In the notes/body field, add allowed apps using categories or specific app names
                  </li>
                  <li>Save the event</li>
                </ol>
                <div className="mt-4 p-3 bg-muted rounded-lg">
                  <p className="text-xs font-medium mb-2">Example:</p>
                  <div className="space-y-1">
                    <p className="text-sm font-mono">🎯 Focus Time: Deep Work</p>
                    <p className="text-xs text-muted-foreground">9:00 AM - 11:00 AM</p>
                    <p className="text-xs text-muted-foreground font-mono mt-2">
                      Notes: @coding @terminal @music
                    </p>
                  </div>
                </div>
              </AccordionContent>
            </AccordionItem>

            <AccordionItem value="format">
              <AccordionTrigger>Description Format Options</AccordionTrigger>
              <AccordionContent className="space-y-3">
                <div className="space-y-3">
                  <div>
                    <p className="text-sm font-medium mb-2">Option 1: Use Categories</p>
                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs font-mono">@coding @communication @music</p>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">
                      Quick setup with predefined app groups
                    </p>
                  </div>

                  <div>
                    <p className="text-sm font-medium mb-2">Option 2: List Specific Apps</p>
                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs font-mono">Visual Studio Code, Slack, Spotify</p>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">
                      Precise control over allowed apps
                    </p>
                  </div>

                  <div>
                    <p className="text-sm font-medium mb-2">Option 3: Mix Categories and Apps</p>
                    <div className="p-3 bg-muted rounded-lg">
                      <p className="text-xs font-mono">@coding @music Notion</p>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">
                      Combine both for flexibility
                    </p>
                  </div>
                </div>
              </AccordionContent>
            </AccordionItem>
          </Accordion>
        </CardContent>
      </Card>

      {/* Available Categories */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="h-5 w-5 text-green-500" />
            Available Categories
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground mb-4">
            Use these predefined categories in your calendar event description. Each category
            includes a curated list of commonly used apps.
          </p>
          <div className="grid gap-4 md:grid-cols-2">
            {(Object.entries(FOCUS_TIME_CATEGORIES) as [string, readonly string[]][]).map(
              ([category, apps]) => (
                <div
                  key={category}
                  className="p-3 border rounded-lg hover:bg-muted/50 transition-colors"
                >
                  <div className="flex items-center justify-between mb-2">
                    <Badge variant="secondary" className="font-mono">
                      {category}
                    </Badge>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 w-6 p-0"
                      onClick={() => copyToClipboard(category, category)}
                    >
                      {copiedText === category ? (
                        <CheckCircle className="h-3 w-3 text-green-500" />
                      ) : (
                        <Copy className="h-3 w-3" />
                      )}
                    </Button>
                  </div>
                  <p className="text-xs text-muted-foreground">{apps.join(", ")}</p>
                </div>
              )
            )}
          </div>
        </CardContent>
      </Card>

      {/* Example Calendar Events */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Lightbulb className="h-5 w-5 text-green-500" />
            Example Calendar Events
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {EXAMPLE_EVENTS.map((event, idx) => (
              <div key={idx} className="p-4 border rounded-lg hover:bg-muted/50 transition-colors">
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1 space-y-2">
                    <div>
                      <p className="font-medium text-sm">{event.title}</p>
                      <p className="text-xs text-muted-foreground flex items-center gap-1 mt-1">
                        <Clock className="h-3 w-3" />
                        {event.time}
                      </p>
                    </div>
                    <div className="p-2 bg-muted rounded text-xs font-mono">
                      Description: {event.description}
                    </div>
                    <p className="text-xs text-muted-foreground italic">{event.useCase}</p>
                    <div className="flex flex-wrap gap-1">
                      <p className="text-xs font-medium text-muted-foreground w-full mb-1">
                        Allowed apps:
                      </p>
                      {event.apps.map((app) => (
                        <Badge key={app} variant="outline" className="text-xs">
                          {app}
                        </Badge>
                      ))}
                    </div>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => copyToClipboard(event.description, `example-${idx}`)}
                  >
                    {copiedText === `example-${idx}` ? (
                      <CheckCircle className="h-4 w-4 text-green-500" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* During Focus Time */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-green-500" />
            During Focus Time
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <p className="text-sm font-medium mb-2">What Happens</p>
            <ul className="space-y-2 ml-4">
              <li className="text-sm flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>A green overlay appears in the top-right corner showing time remaining</span>
              </li>
              <li className="text-sm flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Only apps in your allowed list can be opened</span>
              </li>
              <li className="text-sm flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>Blocked apps will be automatically closed or prevented from launching</span>
              </li>
              <li className="text-sm flex items-start gap-2">
                <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>
                  The overlay is minimal and stays out of your way while keeping you informed
                </span>
              </li>
            </ul>
          </div>

          <div>
            <p className="text-sm font-medium mb-2">Override Options</p>
            <ul className="space-y-2 ml-4">
              <li className="text-sm flex items-start gap-2">
                <Target className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>
                  <strong>Modify Apps:</strong> Click "Modify" in the overlay to add or remove
                  allowed apps during the session
                </span>
              </li>
              <li className="text-sm flex items-start gap-2">
                <Target className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>
                  <strong>End Early:</strong> Click "End" if you need to finish your focus session
                  before the scheduled time
                </span>
              </li>
              <li className="text-sm flex items-start gap-2">
                <Target className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                <span>
                  <strong>Emergency Override:</strong> You always have full control - no permanent
                  locks
                </span>
              </li>
            </ul>
          </div>
        </CardContent>
      </Card>

      {/* Tips & Best Practices */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Lightbulb className="h-5 w-5 text-green-500" />
            Tips & Best Practices
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Accordion type="single" collapsible>
            <AccordionItem value="session-length">
              <AccordionTrigger>Recommended Session Lengths</AccordionTrigger>
              <AccordionContent className="space-y-2">
                <div className="space-y-3">
                  <div className="p-3 border-l-4 border-green-500 bg-muted/50">
                    <p className="text-sm font-medium">Deep Work: 90-120 minutes</p>
                    <p className="text-xs text-muted-foreground mt-1">
                      Ideal for complex tasks requiring sustained concentration
                    </p>
                  </div>
                  <div className="p-3 border-l-4 border-blue-500 bg-muted/50">
                    <p className="text-sm font-medium">Standard Focus: 45-60 minutes</p>
                    <p className="text-xs text-muted-foreground mt-1">
                      Good for most focused work with a break afterward
                    </p>
                  </div>
                  <div className="p-3 border-l-4 border-purple-500 bg-muted/50">
                    <p className="text-sm font-medium">Quick Focus: 25-30 minutes</p>
                    <p className="text-xs text-muted-foreground mt-1">
                      Pomodoro-style sessions for shorter tasks
                    </p>
                  </div>
                </div>
              </AccordionContent>
            </AccordionItem>

            <AccordionItem value="daily-structure">
              <AccordionTrigger>How to Structure Your Day</AccordionTrigger>
              <AccordionContent className="space-y-2">
                <ul className="space-y-2 text-sm">
                  <li className="flex items-start gap-2">
                    <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                    <span>
                      <strong>Morning:</strong> Schedule deep work when energy is highest (9-11 AM)
                    </span>
                  </li>
                  <li className="flex items-start gap-2">
                    <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                    <span>
                      <strong>After Lunch:</strong> Lighter focus work or collaborative tasks (2-4
                      PM)
                    </span>
                  </li>
                  <li className="flex items-start gap-2">
                    <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                    <span>
                      <strong>Breaks:</strong> Schedule 10-15 minute breaks between sessions
                    </span>
                  </li>
                  <li className="flex items-start gap-2">
                    <CheckCircle className="h-4 w-4 text-green-500 mt-0.5 flex-shrink-0" />
                    <span>
                      <strong>Flexibility:</strong> Leave some buffer time for unexpected tasks
                    </span>
                  </li>
                </ul>
              </AccordionContent>
            </AccordionItem>

            <AccordionItem value="category-tips">
              <AccordionTrigger>Choosing the Right Categories</AccordionTrigger>
              <AccordionContent className="space-y-2">
                <ul className="space-y-2 text-sm">
                  <li className="flex items-start gap-2">
                    <AlertCircle className="h-4 w-4 text-blue-500 mt-0.5 flex-shrink-0" />
                    <span>Start broad with categories, then add specific apps if needed</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <AlertCircle className="h-4 w-4 text-blue-500 mt-0.5 flex-shrink-0" />
                    <span>Always include @music if you work better with background audio</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <AlertCircle className="h-4 w-4 text-blue-500 mt-0.5 flex-shrink-0" />
                    <span>Be minimal - only allow what you truly need for the task</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <AlertCircle className="h-4 w-4 text-blue-500 mt-0.5 flex-shrink-0" />
                    <span>Create templates by duplicating successful events in your calendar</span>
                  </li>
                </ul>
              </AccordionContent>
            </AccordionItem>
          </Accordion>
        </CardContent>
      </Card>

      {/* Quick Reference */}
      <Card className="bg-green-500/10 border-green-500/50">
        <CardHeader>
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Target className="h-4 w-4 text-green-500" />
            Quick Reference
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div>
              <p className="text-xs font-medium mb-1">Event Title Must Start With:</p>
              <div className="flex flex-wrap gap-1">
                <Badge variant="outline" className="font-mono">
                  🎯 Focus Time:
                </Badge>
                <Badge variant="outline" className="font-mono">
                  [Focus]
                </Badge>
                <Badge variant="outline" className="font-mono">
                  Focus:
                </Badge>
              </div>
            </div>
            <div>
              <p className="text-xs font-medium mb-1">Description Format:</p>
              <code className="text-xs bg-background p-2 rounded block">
                @category1 @category2 Specific App Name
              </code>
            </div>
            <div>
              <p className="text-xs font-medium mb-1">Most Common Setup:</p>
              <code className="text-xs bg-background p-2 rounded block">
                @coding @terminal @music
              </code>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
