// features/blocking/blocking-schedule.tsx - Schedule-based blocking configuration

import { useState } from "react";
import { Clock, Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { toast } from "sonner";
import {
  useBlockingSchedules,
  useCreateSchedule,
  useUpdateSchedule,
  useDeleteSchedule,
} from "@/hooks/use-blocking-advanced";
import { DAY_NAMES } from "@focusflow/types";

export function BlockingSchedule() {
  const [selectedDay, setSelectedDay] = useState<number>(1); // Monday
  const [startTime, setStartTime] = useState("09:00");
  const [endTime, setEndTime] = useState("17:00");

  const { data: schedules, isLoading } = useBlockingSchedules();
  const createSchedule = useCreateSchedule();
  const updateSchedule = useUpdateSchedule();
  const deleteSchedule = useDeleteSchedule();

  const handleCreateSchedule = async () => {
    try {
      await createSchedule.mutateAsync({
        dayOfWeek: selectedDay,
        startTime,
        endTime,
      });
      toast.success("Schedule created successfully");
    } catch (error) {
      toast.error("Failed to create schedule");
      console.error(error);
    }
  };

  const handleToggleSchedule = async (id: number, enabled: boolean) => {
    try {
      await updateSchedule.mutateAsync({
        id,
        enabled,
      });
      toast.success(`Schedule ${enabled ? "enabled" : "disabled"}`);
    } catch (error) {
      toast.error("Failed to update schedule");
      console.error(error);
    }
  };

  const handleDeleteSchedule = async (id: number) => {
    try {
      await deleteSchedule.mutateAsync(id);
      toast.success("Schedule deleted");
    } catch (error) {
      toast.error("Failed to delete schedule");
      console.error(error);
    }
  };

  // Group schedules by day
  const schedulesByDay = Array.isArray(schedules)
    ? schedules.reduce(
        (acc, schedule) => {
          if (!acc[schedule.dayOfWeek]) {
            acc[schedule.dayOfWeek] = [];
          }
          acc[schedule.dayOfWeek].push(schedule);
          return acc;
        },
        {} as Record<number, typeof schedules>
      )
    : {};

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">Blocking Schedules</h3>
        <p className="text-sm text-muted-foreground">
          Automatically enable blocking during specific times
        </p>
      </div>

      {/* Create Schedule Card */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Add New Schedule</CardTitle>
          <CardDescription>Block sites and apps during work hours automatically</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="space-y-2">
              <Label htmlFor="day">Day of Week</Label>
              <Select
                value={selectedDay.toString()}
                onValueChange={(value: string) => setSelectedDay(parseInt(value))}
              >
                <SelectTrigger id="day">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {DAY_NAMES.map((day, index) => (
                    <SelectItem key={index} value={index.toString()}>
                      {day}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="start-time">Start Time</Label>
              <input
                id="start-time"
                type="time"
                value={startTime}
                onChange={(e) => setStartTime(e.target.value)}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="end-time">End Time</Label>
              <input
                id="end-time"
                type="time"
                value={endTime}
                onChange={(e) => setEndTime(e.target.value)}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
              />
            </div>
          </div>

          <Button
            onClick={handleCreateSchedule}
            disabled={createSchedule.isPending}
            className="w-full"
          >
            <Plus className="h-4 w-4 mr-2" />
            Add Schedule
          </Button>
        </CardContent>
      </Card>

      {/* Existing Schedules */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Active Schedules</CardTitle>
          <CardDescription>
            Manage your automatic blocking schedules
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="text-sm text-muted-foreground">Loading schedules...</div>
          ) : !schedules || schedules.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              No schedules configured. Create one above to get started.
            </div>
          ) : (
            <div className="space-y-4">
              {DAY_NAMES.map((dayName, dayIndex) => {
                const daySchedules = schedulesByDay?.[dayIndex];
                if (!daySchedules || daySchedules.length === 0) return null;

                return (
                  <div key={dayIndex} className="space-y-2">
                    <h4 className="font-medium text-sm">{dayName}</h4>
                    <div className="space-y-2">
                      {daySchedules.map((schedule) => (
                        <div
                          key={schedule.id}
                          className="flex items-center justify-between p-3 rounded-lg border bg-card"
                        >
                          <div className="flex items-center gap-3">
                            <Clock className="h-4 w-4 text-muted-foreground" />
                            <div className="text-sm">
                              {schedule.startTime} - {schedule.endTime}
                            </div>
                          </div>

                          <div className="flex items-center gap-2">
                            <Switch
                              checked={schedule.enabled}
                              onCheckedChange={(checked) =>
                                handleToggleSchedule(schedule.id, checked)
                              }
                            />
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleDeleteSchedule(schedule.id)}
                              className="h-8 w-8 p-0"
                            >
                              <Trash2 className="h-4 w-4 text-destructive" />
                            </Button>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
