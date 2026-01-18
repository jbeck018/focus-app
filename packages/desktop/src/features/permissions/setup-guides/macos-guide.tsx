// features/permissions/setup-guides/macos-guide.tsx - macOS-specific permissions setup guide

import { Terminal, ShieldAlert, Lock, CheckCircle2, Copy, ChevronDown } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { toast } from "sonner";
import { useState } from "react";

interface CodeBlockProps {
  code: string;
  description?: string;
}

function CodeBlock({ code, description }: CodeBlockProps) {
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
        <pre className="bg-muted p-4 rounded-lg text-sm overflow-x-auto border">
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

export function MacOSGuide() {
  return (
    <div className="space-y-6">
      {/* Overview Alert */}
      <Alert>
        <Terminal className="h-4 w-4" />
        <AlertTitle>macOS Permissions Setup</AlertTitle>
        <AlertDescription>
          FocusFlow needs write access to <code className="text-xs bg-muted px-1 rounded">/etc/hosts</code>
          to block websites at the DNS level. Choose one of the methods below based on your security preferences.
        </AlertDescription>
      </Alert>

      {/* Security Warning */}
      <Alert variant="destructive">
        <ShieldAlert className="h-4 w-4" />
        <AlertTitle>Security Considerations</AlertTitle>
        <AlertDescription className="space-y-2">
          <p>Each method has different security implications:</p>
          <ul className="list-disc list-inside text-sm space-y-1 mt-2">
            <li><strong>Temporary Sudo:</strong> Most secure, requires password each launch</li>
            <li><strong>NOPASSWD Sudoers:</strong> Convenient but allows passwordless access to hosts file</li>
            <li><strong>File Permissions:</strong> Least secure, any process can modify hosts file</li>
          </ul>
        </AlertDescription>
      </Alert>

      {/* Method 1: Temporary Sudo (Recommended) */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-green-500/10 text-green-700 dark:text-green-400">
              Recommended
            </Badge>
            <CardTitle className="text-lg">Method 1: Run with Sudo (Temporary)</CardTitle>
          </div>
          <CardDescription>
            Most secure option - enter your password each time you launch FocusFlow
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Open Terminal
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              You can find Terminal in Applications → Utilities, or press Cmd+Space and type "Terminal"
            </p>
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                2
              </span>
              Launch FocusFlow with sudo
            </h4>
            <CodeBlock
              code="sudo /Applications/FocusFlow.app/Contents/MacOS/FocusFlow"
              description="This will prompt for your password, then launch the app with necessary permissions"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                3
              </span>
              Enter your password
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              You'll need to enter your macOS password (the one you use to log in)
            </p>
          </div>

          <Alert>
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertDescription>
              You'll need to repeat this process each time you launch FocusFlow, but your system remains secure.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Method 2: Sudoers Configuration */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-blue-500/10 text-blue-700 dark:text-blue-400">
              Convenient
            </Badge>
            <CardTitle className="text-lg">Method 2: Configure Passwordless Sudo</CardTitle>
          </div>
          <CardDescription>
            Allow FocusFlow to modify hosts file without entering password each time
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert variant="destructive">
            <ShieldAlert className="h-4 w-4" />
            <AlertDescription className="text-sm">
              This method allows the app to modify /etc/hosts without a password. Only use if you trust FocusFlow completely.
            </AlertDescription>
          </Alert>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Open sudoers configuration
            </h4>
            <CodeBlock
              code="sudo visudo"
              description="This opens the sudoers file in a safe editor that prevents syntax errors"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                2
              </span>
              Add the following line
            </h4>
            <CodeBlock
              code={`# Allow FocusFlow to modify hosts file without password
%admin ALL=(ALL) NOPASSWD: /usr/bin/tee /etc/hosts, /bin/cat /etc/hosts`}
              description="Add this to the end of the file. Press 'i' to enter insert mode in vi, paste the text, then press Esc and type ':wq' to save and exit."
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                3
              </span>
              Save and test
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              The changes take effect immediately. Launch FocusFlow normally - no sudo required.
            </p>
          </div>

          <CollapsibleSection title="Advanced: User-specific configuration">
            <p className="text-sm text-muted-foreground">
              If you want to grant access only to your user (not all admin users), replace <code className="text-xs bg-muted px-1 rounded">%admin</code> with your username:
            </p>
            <CodeBlock
              code={`# Replace 'yourusername' with your actual username
yourusername ALL=(ALL) NOPASSWD: /usr/bin/tee /etc/hosts, /bin/cat /etc/hosts`}
            />
            <p className="text-sm text-muted-foreground mt-2">
              To find your username, run: <code className="text-xs bg-muted px-1 rounded">whoami</code>
            </p>
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Method 3: Change File Permissions */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-amber-500/10 text-amber-700 dark:text-amber-400">
              Not Recommended
            </Badge>
            <CardTitle className="text-lg">Method 3: Modify File Permissions</CardTitle>
          </div>
          <CardDescription>
            Make the hosts file writable by your user - convenient but least secure
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert variant="destructive">
            <ShieldAlert className="h-4 w-4" />
            <AlertDescription className="text-sm space-y-2">
              <p className="font-medium">Security Risk:</p>
              <p>
                This makes /etc/hosts writable by your user account, meaning any application you run
                can modify it. Malicious software could add entries without your knowledge.
              </p>
            </AlertDescription>
          </Alert>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Backup the current hosts file
            </h4>
            <CodeBlock
              code="sudo cp /etc/hosts /etc/hosts.backup"
              description="Always create a backup before modifying system files"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                2
              </span>
              Change ownership to your user
            </h4>
            <CodeBlock
              code="sudo chown $(whoami) /etc/hosts"
              description="This makes you the owner of the hosts file"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                3
              </span>
              Set write permissions
            </h4>
            <CodeBlock
              code="sudo chmod 644 /etc/hosts"
              description="Allows you to read and write, others can only read"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                4
              </span>
              Verify permissions
            </h4>
            <CodeBlock
              code="ls -l /etc/hosts"
              description="Should show: -rw-r--r-- 1 yourusername wheel"
            />
          </div>

          <CollapsibleSection title="How to revert these changes">
            <p className="text-sm text-muted-foreground mb-3">
              If you want to restore the original secure permissions:
            </p>
            <CodeBlock
              code={`# Restore original owner and permissions
sudo chown root:wheel /etc/hosts
sudo chmod 644 /etc/hosts

# Restore from backup if needed
sudo cp /etc/hosts.backup /etc/hosts`}
            />
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Troubleshooting */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Lock className="h-5 w-5" />
            Troubleshooting
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <CollapsibleSection title='"Permission denied" error'>
            <p className="text-sm text-muted-foreground">
              If you see "Permission denied" when trying to modify /etc/hosts:
            </p>
            <ul className="list-disc list-inside text-sm space-y-2 mt-2 ml-4">
              <li>Make sure you're using <code className="text-xs bg-muted px-1 rounded">sudo</code> for commands that require it</li>
              <li>Verify your account has admin privileges (System Settings → Users & Groups)</li>
              <li>Try restarting Terminal and running the command again</li>
            </ul>
          </CollapsibleSection>

          <CollapsibleSection title="Blocking still doesn't work after setup">
            <p className="text-sm text-muted-foreground mb-3">
              If websites aren't being blocked even after permissions are set:
            </p>
            <ol className="list-decimal list-inside text-sm space-y-2 ml-4">
              <li>Flush your DNS cache:
                <CodeBlock code="sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder" />
              </li>
              <li>Check if the entries were added to /etc/hosts:
                <CodeBlock code="cat /etc/hosts | grep focusflow" />
              </li>
              <li>Try restarting your browser or the blocked application</li>
              <li>Some browsers use their own DNS (like Brave/Chrome with DNS-over-HTTPS) - disable it in browser settings</li>
            </ol>
          </CollapsibleSection>

          <CollapsibleSection title="App crashes when trying to block">
            <p className="text-sm text-muted-foreground">
              If FocusFlow crashes when attempting to block:
            </p>
            <ul className="list-disc list-inside text-sm space-y-2 mt-2 ml-4">
              <li>Check system logs: Console.app → System Reports</li>
              <li>Verify /etc/hosts is not locked by another process</li>
              <li>Try running with verbose logging to see detailed errors</li>
              <li>Ensure you have enough disk space (check with <code className="text-xs bg-muted px-1 rounded">df -h</code>)</li>
            </ul>
          </CollapsibleSection>

          <CollapsibleSection title="Restore original hosts file">
            <p className="text-sm text-muted-foreground mb-3">
              To restore a clean hosts file:
            </p>
            <CodeBlock
              code={`# Create backup first
sudo cp /etc/hosts /etc/hosts.focusflow-backup

# Restore default macOS hosts file
sudo tee /etc/hosts > /dev/null <<EOF
##
# Host Database
#
# localhost is used to configure the loopback interface
# when the system is booting.  Do not change this entry.
##
127.0.0.1       localhost
255.255.255.255 broadcasthost
::1             localhost
EOF

# Flush DNS cache
sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder`}
            />
          </CollapsibleSection>
        </CardContent>
      </Card>
    </div>
  );
}
