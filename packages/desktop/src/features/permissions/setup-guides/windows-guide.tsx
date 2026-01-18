// features/permissions/setup-guides/windows-guide.tsx - Windows-specific permissions setup guide

import {
  Terminal,
  ShieldAlert,
  Shield,
  CheckCircle2,
  Copy,
  ChevronDown,
  ExternalLink,
} from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { toast } from "sonner";
import { useState } from "react";

interface CodeBlockProps {
  code: string;
  description?: string;
  language?: "powershell" | "batch";
}

function CodeBlock({ code, description, language = "powershell" }: CodeBlockProps) {
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      toast.success("Copied to clipboard");
    } catch (error) {
      toast.error("Failed to copy to clipboard");
      console.error(error);
    }
  };

  return (
    <div className="space-y-2">
      {description && <p className="text-sm text-muted-foreground">{description}</p>}
      <div className="relative group">
        <div className="absolute top-2 left-2 text-xs text-muted-foreground font-mono">
          {language === "powershell" ? "PowerShell" : "CMD"}
        </div>
        <pre className="bg-muted p-4 pt-8 rounded-lg text-sm overflow-x-auto border">
          <code className="font-mono">{code}</code>
        </pre>
        <Button
          size="icon-sm"
          variant="ghost"
          className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity"
          onClick={handleCopy}
          aria-label="Copy code to clipboard"
        >
          <Copy className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

interface CollapsibleSectionProps {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}

function CollapsibleSection({ title, children, defaultOpen = false }: CollapsibleSectionProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <div className="border rounded-lg">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center justify-between p-4 text-left hover:bg-muted/50 transition-colors"
        aria-expanded={isOpen}
      >
        <h4 className="font-medium">{title}</h4>
        <ChevronDown
          className={`h-4 w-4 transition-transform ${isOpen ? "transform rotate-180" : ""}`}
        />
      </button>
      {isOpen && <div className="px-4 pb-4 space-y-4">{children}</div>}
    </div>
  );
}

export function WindowsGuide() {
  return (
    <div className="space-y-6">
      {/* Overview Alert */}
      <Alert>
        <Terminal className="h-4 w-4" />
        <AlertTitle>Windows Permissions Setup</AlertTitle>
        <AlertDescription>
          FocusFlow needs write access to{" "}
          <code className="text-xs bg-muted px-1 rounded">
            C:\Windows\System32\drivers\etc\hosts
          </code>{" "}
          to block websites at the DNS level. Choose one of the methods below.
        </AlertDescription>
      </Alert>

      {/* Security Warning */}
      <Alert variant="destructive">
        <ShieldAlert className="h-4 w-4" />
        <AlertTitle>Administrator Access Required</AlertTitle>
        <AlertDescription className="space-y-2">
          <p>All methods require administrator privileges. Choose based on your preference:</p>
          <ul className="list-disc list-inside text-sm space-y-1 mt-2">
            <li>
              <strong>Run as Admin:</strong> Most secure, prompts UAC each launch
            </li>
            <li>
              <strong>Shortcut with Admin:</strong> Convenient, automatic elevation
            </li>
            <li>
              <strong>File Permissions:</strong> Grant specific permissions to hosts file
            </li>
          </ul>
        </AlertDescription>
      </Alert>

      {/* Method 1: Run as Administrator */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge
              variant="secondary"
              className="bg-green-500/10 text-green-700 dark:text-green-400"
            >
              Recommended
            </Badge>
            <CardTitle className="text-lg">Method 1: Run as Administrator</CardTitle>
          </div>
          <CardDescription>
            Most secure - Windows will prompt for elevation each time you launch
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Locate FocusFlow executable
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Usually found at:{" "}
              <code className="text-xs bg-muted px-1 rounded">
                C:\Program Files\FocusFlow\FocusFlow.exe
              </code>
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                2
              </span>
              Right-click the executable
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Right-click on FocusFlow.exe and select "Run as administrator"
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                3
              </span>
              Accept UAC prompt
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Click "Yes" when Windows User Account Control asks for permission
            </p>
          </div>

          <Alert>
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertDescription>
              You'll need to do this each time you launch FocusFlow. For a more convenient option,
              see Method 2.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Method 2: Create Admin Shortcut */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-blue-500/10 text-blue-700 dark:text-blue-400">
              Convenient
            </Badge>
            <CardTitle className="text-lg">
              Method 2: Create Shortcut with Admin Privileges
            </CardTitle>
          </div>
          <CardDescription>
            Automatically run as administrator - set it once, use it always
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Find the FocusFlow executable
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Navigate to{" "}
              <code className="text-xs bg-muted px-1 rounded">C:\Program Files\FocusFlow\</code>
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                2
              </span>
              Create a shortcut
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Right-click FocusFlow.exe → Send to → Desktop (create shortcut)
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                3
              </span>
              Open shortcut properties
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Right-click the desktop shortcut → Properties
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                4
              </span>
              Enable "Run as administrator"
            </h4>
            <ol className="text-sm text-muted-foreground ml-8 space-y-2 list-decimal list-inside">
              <li>Click the "Advanced..." button</li>
              <li>Check "Run as administrator"</li>
              <li>Click OK twice to save</li>
            </ol>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                5
              </span>
              Pin to taskbar (optional)
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Right-click the shortcut → Pin to taskbar for quick access
            </p>
          </div>

          <Alert>
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertDescription>
              Now you can launch FocusFlow from this shortcut and it will automatically request
              admin privileges.
            </AlertDescription>
          </Alert>

          <CollapsibleSection title="Alternative: Task Scheduler method">
            <p className="text-sm text-muted-foreground mb-3">
              For advanced users, you can create a Task Scheduler task that runs FocusFlow with
              highest privileges without UAC prompts:
            </p>
            <ol className="text-sm space-y-2 ml-4 list-decimal list-inside">
              <li>Open Task Scheduler (search in Start menu)</li>
              <li>Click "Create Task" (not "Create Basic Task")</li>
              <li>Name it "FocusFlow" and check "Run with highest privileges"</li>
              <li>In Actions tab, click "New" and browse to FocusFlow.exe</li>
              <li>
                In Conditions tab, uncheck "Start the task only if the computer is on AC power"
              </li>
              <li>Click OK to save</li>
            </ol>
            <p className="text-sm text-muted-foreground mt-3">
              Create a shortcut to run the task:{" "}
              <code className="text-xs bg-muted px-1 rounded">schtasks /run /tn "FocusFlow"</code>
            </p>
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Method 3: Modify File Permissions */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge
              variant="secondary"
              className="bg-amber-500/10 text-amber-700 dark:text-amber-400"
            >
              Advanced
            </Badge>
            <CardTitle className="text-lg">Method 3: Grant File Permissions</CardTitle>
          </div>
          <CardDescription>
            Give your user account specific permissions to modify the hosts file
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert variant="destructive">
            <ShieldAlert className="h-4 w-4" />
            <AlertDescription className="text-sm">
              This allows any application running under your user account to modify the hosts file.
              Only recommended if you understand the security implications.
            </AlertDescription>
          </Alert>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Navigate to the hosts file
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Open File Explorer and go to:{" "}
              <code className="text-xs bg-muted px-1 rounded">C:\Windows\System32\drivers\etc</code>
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                2
              </span>
              Backup the hosts file
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Right-click "hosts" → Copy, then paste it in the same folder. Windows will create
              "hosts - Copy"
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                3
              </span>
              Open Properties
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Right-click the "hosts" file → Properties
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                4
              </span>
              Modify permissions
            </h4>
            <ol className="text-sm text-muted-foreground ml-8 space-y-2 list-decimal list-inside">
              <li>Go to the "Security" tab</li>
              <li>Click "Edit" button</li>
              <li>Select your user account from the list</li>
              <li>Check "Full control" in the Allow column</li>
              <li>Click Apply, then OK</li>
              <li>Click OK on the Properties window</li>
            </ol>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                5
              </span>
              Test the changes
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Try opening the hosts file with Notepad - you should be able to edit and save it
              without admin privileges
            </p>
          </div>

          <CollapsibleSection title="PowerShell method (faster)">
            <p className="text-sm text-muted-foreground mb-3">
              Open PowerShell as Administrator and run:
            </p>
            <CodeBlock
              code={`# Backup hosts file
Copy-Item C:\\Windows\\System32\\drivers\\etc\\hosts C:\\Windows\\System32\\drivers\\etc\\hosts.backup

# Grant current user full control
$hostsPath = "C:\\Windows\\System32\\drivers\\etc\\hosts"
$acl = Get-Acl $hostsPath
$permission = "$env:USERNAME","FullControl","Allow"
$accessRule = New-Object System.Security.AccessControl.FileSystemAccessRule $permission
$acl.SetAccessRule($accessRule)
Set-Acl $hostsPath $acl

Write-Host "Permissions granted to $env:USERNAME" -ForegroundColor Green`}
              language="powershell"
            />
          </CollapsibleSection>

          <CollapsibleSection title="How to revert permissions">
            <p className="text-sm text-muted-foreground mb-3">
              To restore the original secure permissions:
            </p>
            <CodeBlock
              code={`# Open PowerShell as Administrator and run:
$hostsPath = "C:\\Windows\\System32\\drivers\\etc\\hosts"
$acl = Get-Acl $hostsPath

# Remove the permission you added
$acl.Access | Where-Object { $_.IdentityReference -eq "$env:USERDOMAIN\\$env:USERNAME" } | ForEach-Object {
    $acl.RemoveAccessRule($_)
}

Set-Acl $hostsPath $acl

# Restore from backup if needed
Copy-Item C:\\Windows\\System32\\drivers\\etc\\hosts.backup C:\\Windows\\System32\\drivers\\etc\\hosts -Force

Write-Host "Permissions reverted" -ForegroundColor Green`}
              language="powershell"
            />
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Troubleshooting */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Troubleshooting
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <CollapsibleSection title='"Access Denied" error'>
            <p className="text-sm text-muted-foreground mb-3">
              If you get "Access Denied" when trying to modify the hosts file:
            </p>
            <ul className="list-disc list-inside text-sm space-y-2 ml-4">
              <li>Make sure you're running as Administrator</li>
              <li>
                Check if antivirus software is protecting the hosts file (temporarily disable
                protection)
              </li>
              <li>Verify the file isn't marked as Read-only (Properties → uncheck Read-only)</li>
              <li>
                Close any programs that might have the hosts file open (browsers, security software)
              </li>
            </ul>
          </CollapsibleSection>

          <CollapsibleSection title="Websites still not blocked">
            <p className="text-sm text-muted-foreground mb-3">
              If blocking isn't working even after setup:
            </p>
            <ol className="list-decimal list-inside text-sm space-y-2 ml-4">
              <li>
                Flush DNS cache (open CMD as Admin):
                <CodeBlock code="ipconfig /flushdns" language="batch" />
              </li>
              <li>
                Verify entries in hosts file (open hosts in Notepad and check for FocusFlow entries)
              </li>
              <li>Restart your browser completely</li>
              <li>
                Check if browser is using DNS-over-HTTPS:
                <ul className="list-disc list-inside ml-4 mt-2 space-y-1">
                  <li>
                    <strong>Chrome/Edge:</strong> Settings → Privacy → Security → Use secure DNS
                    (disable)
                  </li>
                  <li>
                    <strong>Firefox:</strong> Settings → General → Network Settings → Enable DNS
                    over HTTPS (disable)
                  </li>
                </ul>
              </li>
            </ol>
          </CollapsibleSection>

          <CollapsibleSection title="UAC prompts are annoying">
            <Alert className="mb-3">
              <ShieldAlert className="h-4 w-4" />
              <AlertDescription className="text-sm">
                Disabling UAC is not recommended as it reduces system security. Consider using
                Method 2 (shortcut) or the Task Scheduler approach instead.
              </AlertDescription>
            </Alert>
            <p className="text-sm text-muted-foreground">
              If you absolutely need to reduce UAC prompts, use the Task Scheduler method described
              in Method 2's advanced section. This is safer than fully disabling UAC.
            </p>
          </CollapsibleSection>

          <CollapsibleSection title="Antivirus blocking FocusFlow">
            <p className="text-sm text-muted-foreground mb-3">
              Some antivirus software may flag hosts file modifications:
            </p>
            <ul className="list-disc list-inside text-sm space-y-2 ml-4">
              <li>Add FocusFlow.exe to your antivirus exclusion list</li>
              <li>
                Add{" "}
                <code className="text-xs bg-muted px-1 rounded">
                  C:\Windows\System32\drivers\etc\hosts
                </code>{" "}
                to exclusions
              </li>
              <li>
                Windows Defender: Settings → Virus & threat protection → Manage settings →
                Exclusions
              </li>
              <li>Check Event Viewer for blocked actions: Windows Logs → Security</li>
            </ul>
          </CollapsibleSection>

          <CollapsibleSection title="Restore default hosts file">
            <p className="text-sm text-muted-foreground mb-3">
              To restore a clean Windows hosts file:
            </p>
            <CodeBlock
              code={`# Open Notepad as Administrator and save this as C:\\Windows\\System32\\drivers\\etc\\hosts
# Copyright (c) 1993-2009 Microsoft Corp.
#
# This is a sample HOSTS file used by Microsoft TCP/IP for Windows.
#
# This file contains the mappings of IP addresses to host names. Each
# entry should be kept on an individual line. The IP address should
# be placed in the first column followed by the corresponding host name.
# The IP address and the host name should be separated by at least one
# space.
#
# Additionally, comments (such as these) may be inserted on individual
# lines or following the machine name denoted by a '#' symbol.
#
# For example:
#
#      102.54.94.97     rhino.acme.com          # source server
#       38.25.63.10     x.acme.com              # x client host

# localhost name resolution is handled within DNS itself.
#       127.0.0.1       localhost
#       ::1             localhost`}
              language="batch"
            />
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Additional Resources */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <ExternalLink className="h-5 w-5" />
            Additional Resources
          </CardTitle>
        </CardHeader>
        <CardContent>
          <ul className="space-y-2 text-sm">
            <li className="flex items-start gap-2">
              <span className="text-muted-foreground">•</span>
              <span>
                <a
                  href="https://learn.microsoft.com/en-us/troubleshoot/windows-server/networking/modify-hosts-file"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  Microsoft: How to modify the hosts file
                </a>
              </span>
            </li>
            <li className="flex items-start gap-2">
              <span className="text-muted-foreground">•</span>
              <span>
                <a
                  href="https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  Understanding User Account Control (UAC)
                </a>
              </span>
            </li>
          </ul>
        </CardContent>
      </Card>
    </div>
  );
}
