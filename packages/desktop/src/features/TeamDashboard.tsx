// features/TeamDashboard.tsx - Team collaboration dashboard

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Switch } from "@/components/ui/switch";
import {
  useCurrentTeam,
  useTeamStats,
  useTeamBlocklist,
  useTeamMembers,
  useTeamPrivacySettings,
  useCreateTeam,
  useJoinTeam,
  useLeaveTeam,
  useAddTeamBlockedItem,
  useRemoveTeamBlockedItem,
  useUpdateTeamPrivacySettings,
  useSyncTeamBlocklist,
} from "@/hooks/useTeam";
import { ROLE_INFO } from "@focusflow/types";
import type { TeamPrivacySettings } from "@focusflow/types";
import {
  Users,
  UserPlus,
  Shield,
  Copy,
  Check,
  Loader2,
  LogOut,
  Plus,
  Trash2,
  RefreshCw,
  Lock,
} from "lucide-react";

export function TeamDashboard() {
  const { data: team, isLoading } = useCurrentTeam();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!team) {
    return <NoTeamView />;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <Users className="h-5 w-5" />
            {team.name}
          </h2>
          <p className="text-sm text-muted-foreground">
            {team.member_count} member{team.member_count !== 1 ? "s" : ""}
          </p>
        </div>
        <InviteCodeBadge code={team.invite_code} />
      </div>

      <Tabs defaultValue="stats">
        <TabsList>
          <TabsTrigger value="stats">Team Stats</TabsTrigger>
          <TabsTrigger value="blocklist">Shared Blocklist</TabsTrigger>
          <TabsTrigger value="members">Members</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
        </TabsList>

        <TabsContent value="stats" className="mt-4">
          <TeamStatsView />
        </TabsContent>

        <TabsContent value="blocklist" className="mt-4">
          <TeamBlocklistView />
        </TabsContent>

        <TabsContent value="members" className="mt-4">
          <TeamMembersView />
        </TabsContent>

        <TabsContent value="settings" className="mt-4">
          <TeamSettingsView />
        </TabsContent>
      </Tabs>
    </div>
  );
}

function NoTeamView() {
  const [showCreate, setShowCreate] = useState(false);
  const [showJoin, setShowJoin] = useState(false);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <Users className="h-5 w-5" />
          Team
        </h2>
        <p className="text-sm text-muted-foreground">
          Collaborate with your team on focus goals
        </p>
      </div>

      <Card>
        <CardContent className="py-8 text-center">
          <Users className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-lg font-medium mb-2">No team yet</p>
          <p className="text-sm text-muted-foreground mb-6">
            Create a team or join an existing one to share blocklists and track
            team progress together.
          </p>
          <div className="flex justify-center gap-3">
            <Button onClick={() => setShowCreate(true)}>
              <Plus className="h-4 w-4 mr-2" />
              Create Team
            </Button>
            <Button variant="outline" onClick={() => setShowJoin(true)}>
              <UserPlus className="h-4 w-4 mr-2" />
              Join Team
            </Button>
          </div>
        </CardContent>
      </Card>

      <CreateTeamDialog open={showCreate} onOpenChange={setShowCreate} />
      <JoinTeamDialog open={showJoin} onOpenChange={setShowJoin} />
    </div>
  );
}

function CreateTeamDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [name, setName] = useState("");
  const createTeam = useCreateTeam();

  const handleCreate = async () => {
    if (!name.trim()) return;
    await createTeam.mutateAsync(name.trim());
    setName("");
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create a Team</DialogTitle>
          <DialogDescription>
            Create a new team to collaborate on focus goals with others.
          </DialogDescription>
        </DialogHeader>
        <div className="py-4">
          <Input
            placeholder="Team name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleCreate}
            disabled={!name.trim() || createTeam.isPending}
          >
            {createTeam.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              "Create"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function JoinTeamDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [code, setCode] = useState("");
  const joinTeam = useJoinTeam();

  const handleJoin = async () => {
    if (!code.trim()) return;
    await joinTeam.mutateAsync(code.trim());
    setCode("");
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Join a Team</DialogTitle>
          <DialogDescription>
            Enter the invite code to join an existing team.
          </DialogDescription>
        </DialogHeader>
        <div className="py-4">
          <Input
            placeholder="FOCUS-XXXX"
            value={code}
            onChange={(e) => setCode(e.target.value.toUpperCase())}
          />
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleJoin}
            disabled={!code.trim() || joinTeam.isPending}
          >
            {joinTeam.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              "Join"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function InviteCodeBadge({ code }: { code: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Button variant="outline" size="sm" onClick={handleCopy}>
      {copied ? (
        <>
          <Check className="h-4 w-4 mr-1 text-green-500" />
          Copied!
        </>
      ) : (
        <>
          <Copy className="h-4 w-4 mr-1" />
          {code}
        </>
      )}
    </Button>
  );
}

function TeamStatsView() {
  const { data: stats, isLoading } = useTeamStats();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!stats) return null;

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">Team Focus Time</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {stats.total_focus_hours_this_week.toFixed(1)}h
          </div>
          <p className="text-xs text-muted-foreground">This week</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">Avg Sessions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {stats.average_sessions_per_member.toFixed(1)}
          </div>
          <p className="text-xs text-muted-foreground">Per member</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">Most Productive</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {stats.most_productive_day ?? "N/A"}
          </div>
          <p className="text-xs text-muted-foreground">Best day this week</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">Team Size</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{stats.member_count}</div>
          <p className="text-xs text-muted-foreground">Members</p>
        </CardContent>
      </Card>

      {stats.top_blockers.length > 0 && (
        <Card className="md:col-span-2 lg:col-span-4">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Shield className="h-4 w-4" />
              Top Blocked Items
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2">
              {stats.top_blockers.map((item, idx) => (
                <Badge key={idx} variant="secondary">
                  {item}
                </Badge>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function TeamBlocklistView() {
  const { data: blocklist, isLoading } = useTeamBlocklist();
  const [newItem, setNewItem] = useState("");
  const [itemType, setItemType] = useState<"app" | "website">("website");
  const addItem = useAddTeamBlockedItem();
  const removeItem = useRemoveTeamBlockedItem();
  const syncBlocklist = useSyncTeamBlocklist();

  const handleAdd = async () => {
    if (!newItem.trim()) return;
    await addItem.mutateAsync({ itemType, value: newItem.trim() });
    setNewItem("");
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
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Shared blocklist applies to all team members who sync it.
        </p>
        <Button
          variant="outline"
          size="sm"
          onClick={() => syncBlocklist.mutate()}
          disabled={syncBlocklist.isPending}
        >
          {syncBlocklist.isPending ? (
            <Loader2 className="h-4 w-4 animate-spin mr-1" />
          ) : (
            <RefreshCw className="h-4 w-4 mr-1" />
          )}
          Sync to Local
        </Button>
      </div>

      {/* Add new item */}
      <Card>
        <CardContent className="pt-4">
          <div className="flex gap-2">
            <select
              className="px-3 py-2 border rounded-md text-sm"
              value={itemType}
              onChange={(e) => setItemType(e.target.value as "app" | "website")}
            >
              <option value="website">Website</option>
              <option value="app">App</option>
            </select>
            <Input
              placeholder={itemType === "website" ? "example.com" : "App Name"}
              value={newItem}
              onChange={(e) => setNewItem(e.target.value)}
              className="flex-1"
            />
            <Button onClick={handleAdd} disabled={!newItem.trim() || addItem.isPending}>
              {addItem.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Plus className="h-4 w-4" />
              )}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Blocklist items */}
      {blocklist && blocklist.length > 0 ? (
        <div className="space-y-2">
          {blocklist.map((item) => (
            <Card key={item.id}>
              <CardContent className="py-3 flex items-center justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <Badge variant="outline">{item.item_type}</Badge>
                    <span className="font-medium">{item.value}</span>
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    Added by {item.added_by}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => removeItem.mutate(item.id)}
                  disabled={removeItem.isPending}
                >
                  <Trash2 className="h-4 w-4 text-destructive" />
                </Button>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="py-8 text-center">
            <Shield className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <p className="text-muted-foreground">No shared blocklist items yet</p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function TeamMembersView() {
  const { data: members, isLoading } = useTeamMembers();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {members?.map((member) => {
        const roleInfo = ROLE_INFO[member.role];

        return (
          <Card key={member.id}>
            <CardContent className="py-4 flex items-center justify-between">
              <div>
                <div className="flex items-center gap-2">
                  <span className="font-medium">
                    {member.display_name ?? member.email}
                  </span>
                  <Badge variant="outline">{roleInfo.label}</Badge>
                  {member.sharing_enabled && (
                    <Badge variant="secondary" className="text-xs">
                      Sharing
                    </Badge>
                  )}
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  {member.email}
                </p>
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}

function TeamSettingsView() {
  const { data: privacy } = useTeamPrivacySettings();
  const updatePrivacy = useUpdateTeamPrivacySettings();
  const leaveTeam = useLeaveTeam();

  const handlePrivacyChange = (key: keyof TeamPrivacySettings, value: boolean) => {
    if (!privacy) return;
    updatePrivacy.mutate({ ...privacy, [key]: value });
  };

  return (
    <div className="space-y-6">
      {/* Privacy Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <Lock className="h-4 w-4" />
            Privacy Settings
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground">
            Control what data you share with your team. Your individual session
            details are never shared.
          </p>

          <div className="space-y-3">
            <PrivacyToggle
              label="Share focus time"
              description="Contribute your focus hours to team totals"
              checked={privacy?.share_focus_time ?? true}
              onCheckedChange={(v) => handlePrivacyChange("share_focus_time", v)}
            />
            <PrivacyToggle
              label="Share session count"
              description="Include your session count in team averages"
              checked={privacy?.share_session_count ?? true}
              onCheckedChange={(v) => handlePrivacyChange("share_session_count", v)}
            />
            <PrivacyToggle
              label="Share streak"
              description="Show your streak on team leaderboard"
              checked={privacy?.share_streak ?? false}
              onCheckedChange={(v) => handlePrivacyChange("share_streak", v)}
            />
            <PrivacyToggle
              label="Share productivity score"
              description="Include your score in team insights"
              checked={privacy?.share_productivity_score ?? false}
              onCheckedChange={(v) => handlePrivacyChange("share_productivity_score", v)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Leave Team */}
      <Card className="border-destructive/50">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-destructive">
            Danger Zone
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground mb-4">
            Leave this team. You can rejoin later with an invite code.
          </p>
          <Button
            variant="destructive"
            onClick={() => leaveTeam.mutate()}
            disabled={leaveTeam.isPending}
          >
            {leaveTeam.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
            ) : (
              <LogOut className="h-4 w-4 mr-2" />
            )}
            Leave Team
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}

function PrivacyToggle({
  label,
  description,
  checked,
  onCheckedChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="text-sm font-medium">{label}</p>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} />
    </div>
  );
}
