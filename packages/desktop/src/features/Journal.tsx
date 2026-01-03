// features/Journal.tsx - Trigger journaling component

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  useRecentJournalEntries,
  useTriggerInsights,
  usePeakTimes,
  useCreateJournalEntry,
} from "@/hooks/useJournal";
import { useSessionStore } from "@/stores/sessionStore";
import type { TriggerType, Emotion } from "@focusflow/types";
import { TRIGGER_INFO, EMOTION_INFO, DAY_NAMES } from "@focusflow/types";
import { BookOpen, TrendingUp, Clock, Loader2, Plus } from "lucide-react";

export function Journal() {
  const [showNewEntry, setShowNewEntry] = useState(false);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <BookOpen className="h-5 w-5" />
            Trigger Journal
          </h2>
          <p className="text-sm text-muted-foreground">
            Track what triggers distractions to build better focus habits
          </p>
        </div>
        <Button onClick={() => setShowNewEntry(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Log Trigger
        </Button>
      </div>

      <Tabs defaultValue="insights">
        <TabsList>
          <TabsTrigger value="insights">Insights</TabsTrigger>
          <TabsTrigger value="history">History</TabsTrigger>
        </TabsList>

        <TabsContent value="insights" className="space-y-4 mt-4">
          <InsightsView />
        </TabsContent>

        <TabsContent value="history" className="mt-4">
          <HistoryView />
        </TabsContent>
      </Tabs>

      <NewEntryDialog open={showNewEntry} onOpenChange={setShowNewEntry} />
    </div>
  );
}

function InsightsView() {
  const { data: insights, isLoading: insightsLoading } = useTriggerInsights();
  const { data: peakTimes, isLoading: peakLoading } = usePeakTimes();

  if (insightsLoading || peakLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  const totalTriggers = insights?.reduce((sum, i) => sum + i.frequency, 0) ?? 0;

  return (
    <div className="grid gap-4 md:grid-cols-2">
      {/* Top Triggers */}
      <Card>
        <CardHeader>
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <TrendingUp className="h-4 w-4" />
            Top Triggers (30 days)
          </CardTitle>
        </CardHeader>
        <CardContent>
          {insights && insights.length > 0 ? (
            <div className="space-y-3">
              {insights.slice(0, 5).map((insight) => {
                const info = TRIGGER_INFO[insight.trigger_type as TriggerType];
                const percentage = totalTriggers > 0
                  ? Math.round((insight.frequency / totalTriggers) * 100)
                  : 0;

                return (
                  <div key={insight.trigger_type} className="flex items-center gap-3">
                    <span className="text-xl">{info?.emoji ?? "❓"}</span>
                    <div className="flex-1">
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-sm font-medium">
                          {info?.label ?? insight.trigger_type}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {insight.frequency} ({percentage}%)
                        </span>
                      </div>
                      <div className="h-2 bg-secondary rounded-full overflow-hidden">
                        <div
                          className="h-full bg-primary transition-all"
                          style={{ width: `${percentage}%` }}
                        />
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground text-center py-4">
              No triggers logged yet. Start journaling to see insights.
            </p>
          )}
        </CardContent>
      </Card>

      {/* Peak Times */}
      <Card>
        <CardHeader>
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Clock className="h-4 w-4" />
            Peak Distraction Times
          </CardTitle>
        </CardHeader>
        <CardContent>
          {peakTimes && (peakTimes.peak_hour !== null || peakTimes.peak_day !== null) ? (
            <div className="space-y-4">
              {peakTimes.peak_hour !== null && (
                <div className="p-3 bg-secondary/50 rounded-lg">
                  <p className="text-sm text-muted-foreground">Most distracting hour</p>
                  <p className="text-lg font-medium">
                    {formatHour(peakTimes.peak_hour)}
                  </p>
                </div>
              )}
              {peakTimes.peak_day !== null && (
                <div className="p-3 bg-secondary/50 rounded-lg">
                  <p className="text-sm text-muted-foreground">Most distracting day</p>
                  <p className="text-lg font-medium">
                    {DAY_NAMES[peakTimes.peak_day]}
                  </p>
                </div>
              )}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground text-center py-4">
              Log more triggers to discover your peak distraction times.
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function HistoryView() {
  const { data: entries, isLoading } = useRecentJournalEntries(20);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!entries || entries.length === 0) {
    return (
      <Card>
        <CardContent className="py-8 text-center">
          <BookOpen className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground">No journal entries yet</p>
          <p className="text-sm text-muted-foreground mt-1">
            Start logging triggers to build self-awareness
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-3">
      {entries.map((entry) => {
        const triggerInfo = TRIGGER_INFO[entry.trigger_type as TriggerType];
        const emotionInfo = entry.emotion ? EMOTION_INFO[entry.emotion as Emotion] : null;

        return (
          <Card key={entry.id}>
            <CardContent className="py-4">
              <div className="flex items-start gap-3">
                <span className="text-2xl">{triggerInfo?.emoji ?? "❓"}</span>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-medium">
                      {triggerInfo?.label ?? entry.trigger_type}
                    </span>
                    {emotionInfo && (
                      <span className="text-sm text-muted-foreground">
                        • {emotionInfo.emoji} {emotionInfo.label}
                      </span>
                    )}
                    {entry.intensity && (
                      <span className="text-xs px-2 py-0.5 bg-secondary rounded-full">
                        Intensity: {entry.intensity}/5
                      </span>
                    )}
                  </div>
                  {entry.notes && (
                    <p className="text-sm text-muted-foreground line-clamp-2">
                      {entry.notes}
                    </p>
                  )}
                  <p className="text-xs text-muted-foreground mt-2">
                    {formatDate(entry.created_at)}
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}

function NewEntryDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [triggerType, setTriggerType] = useState<TriggerType | null>(null);
  const [emotion, setEmotion] = useState<Emotion | null>(null);
  const [notes, setNotes] = useState("");
  const [intensity, setIntensity] = useState<number>(3);

  const activeSession = useSessionStore((s) => s.activeSession);
  const createEntry = useCreateJournalEntry();

  const handleSubmit = async () => {
    if (!triggerType) return;

    await createEntry.mutateAsync({
      session_id: activeSession?.id,
      trigger_type: triggerType,
      emotion: emotion ?? undefined,
      notes: notes.trim() || undefined,
      intensity,
    });

    // Reset form
    setTriggerType(null);
    setEmotion(null);
    setNotes("");
    setIntensity(3);
    onOpenChange(false);
  };

  const internalTriggers: TriggerType[] = ["boredom", "anxiety", "stress", "fatigue"];
  const externalTriggers: TriggerType[] = ["notification", "person", "environment", "other"];
  const emotions: Emotion[] = ["frustrated", "anxious", "tired", "distracted", "curious", "bored", "overwhelmed", "neutral"];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Log a Trigger</DialogTitle>
          <DialogDescription>
            What caused you to lose focus?
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Trigger Selection */}
          <div className="space-y-2">
            <p className="text-sm font-medium">Internal Triggers</p>
            <div className="flex flex-wrap gap-2">
              {internalTriggers.map((t) => (
                <TriggerButton
                  key={t}
                  trigger={t}
                  selected={triggerType === t}
                  onClick={() => setTriggerType(t)}
                />
              ))}
            </div>
          </div>

          <div className="space-y-2">
            <p className="text-sm font-medium">External Triggers</p>
            <div className="flex flex-wrap gap-2">
              {externalTriggers.map((t) => (
                <TriggerButton
                  key={t}
                  trigger={t}
                  selected={triggerType === t}
                  onClick={() => setTriggerType(t)}
                />
              ))}
            </div>
          </div>

          {/* Emotion Selection */}
          {triggerType && (
            <div className="space-y-2">
              <p className="text-sm font-medium">How are you feeling?</p>
              <div className="flex flex-wrap gap-2">
                {emotions.map((e) => (
                  <EmotionButton
                    key={e}
                    emotion={e}
                    selected={emotion === e}
                    onClick={() => setEmotion(emotion === e ? null : e)}
                  />
                ))}
              </div>
            </div>
          )}

          {/* Intensity */}
          {triggerType && (
            <div className="space-y-2">
              <p className="text-sm font-medium">Intensity: {intensity}/5</p>
              <div className="flex gap-2">
                {[1, 2, 3, 4, 5].map((i) => (
                  <Button
                    key={i}
                    variant={intensity === i ? "default" : "outline"}
                    size="sm"
                    onClick={() => setIntensity(i)}
                    className="w-10 h-10"
                  >
                    {i}
                  </Button>
                ))}
              </div>
            </div>
          )}

          {/* Notes */}
          {triggerType && (
            <div className="space-y-2">
              <p className="text-sm font-medium">Notes (optional)</p>
              <Textarea
                placeholder="What happened? What can you do differently next time?"
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
                rows={3}
              />
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={!triggerType || createEntry.isPending}
          >
            {createEntry.isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : (
              "Save Entry"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function TriggerButton({
  trigger,
  selected,
  onClick,
}: {
  trigger: TriggerType;
  selected: boolean;
  onClick: () => void;
}) {
  const info = TRIGGER_INFO[trigger];
  return (
    <Button
      variant={selected ? "default" : "outline"}
      size="sm"
      onClick={onClick}
      className="gap-1"
    >
      <span>{info.emoji}</span>
      <span>{info.label}</span>
    </Button>
  );
}

function EmotionButton({
  emotion,
  selected,
  onClick,
}: {
  emotion: Emotion;
  selected: boolean;
  onClick: () => void;
}) {
  const info = EMOTION_INFO[emotion];
  return (
    <Button
      variant={selected ? "default" : "outline"}
      size="sm"
      onClick={onClick}
      className="gap-1"
    >
      <span>{info.emoji}</span>
      <span>{info.label}</span>
    </Button>
  );
}

function formatHour(hour: number): string {
  const period = hour >= 12 ? "PM" : "AM";
  const displayHour = hour === 0 ? 12 : hour > 12 ? hour - 12 : hour;
  return `${displayHour}:00 ${period}`;
}

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  return date.toLocaleDateString();
}
