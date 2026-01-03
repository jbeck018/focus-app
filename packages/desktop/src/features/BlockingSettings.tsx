// features/BlockingSettings.tsx - App and website blocking configuration

import { useState } from "react";
import { Plus, X, Monitor, Globe } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { toast } from "sonner";
import {
  useBlockedItems,
  useAddBlockedApp,
  useRemoveBlockedApp,
  useAddBlockedWebsite,
  useRemoveBlockedWebsite,
} from "@/hooks/useTauriCommands";

export function BlockingSettings() {
  const [newApp, setNewApp] = useState("");
  const [newWebsite, setNewWebsite] = useState("");

  const { data: blockedItems, isLoading } = useBlockedItems();
  const addApp = useAddBlockedApp();
  const removeApp = useRemoveBlockedApp();
  const addWebsite = useAddBlockedWebsite();
  const removeWebsite = useRemoveBlockedWebsite();

  const apps = blockedItems?.apps ?? [];
  const websites = blockedItems?.websites ?? [];

  const handleAddApp = async () => {
    if (!newApp.trim()) return;

    try {
      await addApp.mutateAsync(newApp.trim());
      setNewApp("");
      toast.success(`Added "${newApp}" to blocked apps`);
    } catch {
      toast.error("Failed to add blocked app");
    }
  };

  const handleRemoveApp = async (name: string) => {
    try {
      await removeApp.mutateAsync(name);
      toast.success(`Removed "${name}" from blocked apps`);
    } catch {
      toast.error("Failed to remove blocked app");
    }
  };

  const handleAddWebsite = async () => {
    if (!newWebsite.trim()) return;

    // Clean up URL to get domain
    let domain = newWebsite.trim().toLowerCase();
    domain = domain.replace(/^(https?:\/\/)?(www\.)?/, "");
    domain = domain.split("/")[0];

    try {
      await addWebsite.mutateAsync(domain);
      setNewWebsite("");
      toast.success(`Added "${domain}" to blocked websites`);
    } catch {
      toast.error("Failed to add blocked website");
    }
  };

  const handleRemoveWebsite = async (domain: string) => {
    try {
      await removeWebsite.mutateAsync(domain);
      toast.success(`Removed "${domain}" from blocked websites`);
    } catch {
      toast.error("Failed to remove blocked website");
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Distraction Blocking</h2>
        <p className="text-muted-foreground">
          Block distracting apps and websites during focus sessions
        </p>
      </div>

      <Tabs defaultValue="apps">
        <TabsList>
          <TabsTrigger value="apps" className="flex items-center gap-2">
            <Monitor className="h-4 w-4" />
            Apps
          </TabsTrigger>
          <TabsTrigger value="websites" className="flex items-center gap-2">
            <Globe className="h-4 w-4" />
            Websites
          </TabsTrigger>
        </TabsList>

        {/* Apps Tab */}
        <TabsContent value="apps" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Blocked Applications</CardTitle>
              <CardDescription>
                These apps will be closed when you start a focus session
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Add App Form */}
              <div className="flex gap-2">
                <Input
                  placeholder="Enter app name (e.g., Slack, Discord)"
                  value={newApp}
                  onChange={(e) => setNewApp(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleAddApp()}
                />
                <Button onClick={handleAddApp} disabled={!newApp.trim() || addApp.isPending}>
                  <Plus className="h-4 w-4" />
                </Button>
              </div>

              {/* Apps List */}
              <div className="flex flex-wrap gap-2">
                {isLoading ? (
                  <p className="text-sm text-muted-foreground">Loading...</p>
                ) : apps.length === 0 ? (
                  <p className="text-sm text-muted-foreground">
                    No apps blocked yet. Add some to get started.
                  </p>
                ) : (
                  apps.map((app) => (
                    <Badge
                      key={app}
                      variant="secondary"
                      className="flex items-center gap-1 pr-1"
                    >
                      {app}
                      <button
                        onClick={() => handleRemoveApp(app)}
                        className="ml-1 rounded-full p-0.5 hover:bg-muted"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))
                )}
              </div>
            </CardContent>
          </Card>

          {/* Common Apps Suggestions */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Quick Add</CardTitle>
              <CardDescription>Click to add commonly blocked apps</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {["Slack", "Discord", "Telegram", "Messages", "Twitter", "Spotify"].map((app) => (
                  <Button
                    key={app}
                    variant="outline"
                    size="sm"
                    onClick={() => addApp.mutate(app)}
                    disabled={apps.some((a) => a.toLowerCase() === app.toLowerCase())}
                  >
                    {app}
                  </Button>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Websites Tab */}
        <TabsContent value="websites" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Blocked Websites</CardTitle>
              <CardDescription>
                These websites will be blocked in your browser during focus sessions
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Add Website Form */}
              <div className="flex gap-2">
                <Input
                  placeholder="Enter website (e.g., twitter.com)"
                  value={newWebsite}
                  onChange={(e) => setNewWebsite(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleAddWebsite()}
                />
                <Button
                  onClick={handleAddWebsite}
                  disabled={!newWebsite.trim() || addWebsite.isPending}
                >
                  <Plus className="h-4 w-4" />
                </Button>
              </div>

              {/* Websites List */}
              <div className="flex flex-wrap gap-2">
                {isLoading ? (
                  <p className="text-sm text-muted-foreground">Loading...</p>
                ) : websites.length === 0 ? (
                  <p className="text-sm text-muted-foreground">
                    No websites blocked yet. Add some to get started.
                  </p>
                ) : (
                  websites.map((site) => (
                    <Badge
                      key={site}
                      variant="secondary"
                      className="flex items-center gap-1 pr-1"
                    >
                      {site}
                      <button
                        onClick={() => handleRemoveWebsite(site)}
                        className="ml-1 rounded-full p-0.5 hover:bg-muted"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))
                )}
              </div>
            </CardContent>
          </Card>

          {/* Common Websites Suggestions */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Quick Add</CardTitle>
              <CardDescription>Click to add commonly blocked websites</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {[
                  "twitter.com",
                  "facebook.com",
                  "reddit.com",
                  "youtube.com",
                  "instagram.com",
                  "tiktok.com",
                ].map((site) => (
                  <Button
                    key={site}
                    variant="outline"
                    size="sm"
                    onClick={() => addWebsite.mutate(site)}
                    disabled={websites.some((w) => w.toLowerCase() === site.toLowerCase())}
                  >
                    {site}
                  </Button>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
