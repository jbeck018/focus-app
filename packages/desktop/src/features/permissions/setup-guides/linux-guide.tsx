// features/permissions/setup-guides/linux-guide.tsx - Linux-specific permissions setup guide

import { Terminal, ShieldAlert, Package, CheckCircle2, Copy, ChevronDown, Info } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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

export function LinuxGuide() {
  return (
    <div className="space-y-6">
      {/* Overview Alert */}
      <Alert>
        <Terminal className="h-4 w-4" />
        <AlertTitle>Linux Permissions Setup</AlertTitle>
        <AlertDescription>
          FocusFlow needs write access to <code className="text-xs bg-muted px-1 rounded">/etc/hosts</code>
          {" "}to block websites at the DNS level. Choose the method that best fits your distribution and security needs.
        </AlertDescription>
      </Alert>

      {/* Distribution Info */}
      <Alert>
        <Package className="h-4 w-4" />
        <AlertTitle>Distribution Compatibility</AlertTitle>
        <AlertDescription>
          These instructions work on most Linux distributions including Ubuntu, Debian, Fedora, Arch, and openSUSE.
          Commands may vary slightly - check your distribution's documentation if needed.
        </AlertDescription>
      </Alert>

      {/* Security Warning */}
      <Alert variant="destructive">
        <ShieldAlert className="h-4 w-4" />
        <AlertTitle>Security Considerations</AlertTitle>
        <AlertDescription className="space-y-2">
          <p>Each method has different security implications:</p>
          <ul className="list-disc list-inside text-sm space-y-1 mt-2">
            <li><strong>Sudo (recommended):</strong> Most secure, requires password each launch</li>
            <li><strong>Polkit rule:</strong> Convenient, allows specific operations without password</li>
            <li><strong>Group permissions:</strong> Flexible but requires careful group management</li>
            <li><strong>File permissions:</strong> Least secure, any process can modify hosts</li>
          </ul>
        </AlertDescription>
      </Alert>

      {/* Method 1: Run with Sudo */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-green-500/10 text-green-700 dark:text-green-400">
              Recommended
            </Badge>
            <CardTitle className="text-lg">Method 1: Run with Sudo</CardTitle>
          </div>
          <CardDescription>
            Most secure option - enter password each time you launch FocusFlow
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Open your terminal
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Use your distribution's terminal application (GNOME Terminal, Konsole, etc.)
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
              code="sudo focusflow"
              description="If installed via package manager, or use the full path to the binary"
            />
            <p className="text-sm text-muted-foreground ml-8">
              Alternative if installed manually:
            </p>
            <CodeBlock
              code="sudo /opt/FocusFlow/focusflow"
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
              Enter your user password when prompted (you won't see characters as you type)
            </p>
          </div>

          <Alert>
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertDescription>
              You'll need to repeat this each time you launch FocusFlow. For a more convenient option, see the methods below.
            </AlertDescription>
          </Alert>

          <CollapsibleSection title="Create a desktop launcher with sudo">
            <p className="text-sm text-muted-foreground mb-3">
              Create a desktop entry that prompts for sudo password:
            </p>
            <CodeBlock
              code={`# Create a desktop entry
cat > ~/.local/share/applications/focusflow-sudo.desktop <<EOF
[Desktop Entry]
Type=Application
Name=FocusFlow (Admin)
Comment=FocusFlow with elevated permissions
Exec=pkexec env DISPLAY=\${DISPLAY} XAUTHORITY=\${XAUTHORITY} /opt/FocusFlow/focusflow
Icon=/opt/FocusFlow/icon.png
Terminal=false
Categories=Productivity;Utility;
EOF

# Make it executable
chmod +x ~/.local/share/applications/focusflow-sudo.desktop

# Update desktop database
update-desktop-database ~/.local/share/applications/`}
              description="This uses pkexec which provides a graphical password prompt"
            />
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Method 2: Polkit Rule */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-blue-500/10 text-blue-700 dark:text-blue-400">
              Convenient
            </Badge>
            <CardTitle className="text-lg">Method 2: Create Polkit Rule</CardTitle>
          </div>
          <CardDescription>
            Allow FocusFlow to modify hosts file without password using Polkit
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription className="text-sm">
              Polkit (PolicyKit) is a modern authorization framework used by most desktop Linux distributions.
              This method is safer than sudoers as it can be more granular and application-specific.
            </AlertDescription>
          </Alert>

          <Tabs defaultValue="systemd" className="w-full">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="systemd">Systemd-based</TabsTrigger>
              <TabsTrigger value="other">Other distributions</TabsTrigger>
            </TabsList>

            <TabsContent value="systemd" className="space-y-4 mt-4">
              <div className="space-y-3">
                <h4 className="font-medium flex items-center gap-2">
                  <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                    1
                  </span>
                  Create a Polkit rule
                </h4>
                <CodeBlock
                  code={`# For Ubuntu/Debian/Fedora with Polkit
sudo tee /etc/polkit-1/rules.d/10-focusflow.rules > /dev/null <<'EOF'
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.policykit.exec" &&
        action.lookup("program") == "/usr/bin/tee" &&
        subject.isInGroup("sudo")) {
        return polkit.Result.YES;
    }
});
EOF

# Set correct permissions
sudo chmod 644 /etc/polkit-1/rules.d/10-focusflow.rules`}
                  description="This allows users in the sudo group to use tee without a password"
                />
              </div>

              <div className="space-y-3">
                <h4 className="font-medium flex items-center gap-2">
                  <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                    2
                  </span>
                  Reload Polkit
                </h4>
                <CodeBlock
                  code="sudo systemctl restart polkit.service"
                  description="Apply the new rule"
                />
              </div>
            </TabsContent>

            <TabsContent value="other" className="space-y-4 mt-4">
              <p className="text-sm text-muted-foreground">
                For distributions using older Polkit versions or different configurations:
              </p>
              <CodeBlock
                code={`# Check your Polkit configuration directory
ls /etc/polkit-1/localauthority/50-local.d/ 2>/dev/null || ls /var/lib/polkit-1/localauthority/50-local.d/

# Create rule (adjust path based on above)
sudo tee /etc/polkit-1/localauthority/50-local.d/10-focusflow.pkla > /dev/null <<'EOF'
[FocusFlow hosts file access]
Identity=unix-user:*
Action=org.freedesktop.policykit.exec
ResultAny=yes
ResultInactive=yes
ResultActive=yes
EOF`}
              />
            </TabsContent>
          </Tabs>

          <Alert variant="destructive">
            <ShieldAlert className="h-4 w-4" />
            <AlertDescription className="text-sm">
              This rule grants broad permissions. For a more secure setup, create a dedicated script and grant
              permissions only to that script instead of general tee access.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Method 3: Group Permissions */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-purple-500/10 text-purple-700 dark:text-purple-400">
              Flexible
            </Badge>
            <CardTitle className="text-lg">Method 3: Create Group with Access</CardTitle>
          </div>
          <CardDescription>
            Create a dedicated group that has permission to modify the hosts file
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Create a backup
            </h4>
            <CodeBlock
              code="sudo cp /etc/hosts /etc/hosts.backup"
              description="Always backup system files before modification"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                2
              </span>
              Create a dedicated group
            </h4>
            <CodeBlock
              code="sudo groupadd focusflow"
              description="Creates a new group specifically for FocusFlow"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                3
              </span>
              Add your user to the group
            </h4>
            <CodeBlock
              code="sudo usermod -a -G focusflow $USER"
              description="Adds your current user to the focusflow group"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                4
              </span>
              Set group ownership and permissions
            </h4>
            <CodeBlock
              code={`# Change group ownership
sudo chgrp focusflow /etc/hosts

# Set permissions (read/write for owner and group, read for others)
sudo chmod 664 /etc/hosts

# Verify
ls -l /etc/hosts`}
              description="Should show: -rw-rw-r-- 1 root focusflow"
            />
          </div>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                5
              </span>
              Log out and back in
            </h4>
            <p className="text-sm text-muted-foreground ml-8">
              Group membership changes require a new login session. You can verify with:
            </p>
            <CodeBlock
              code="groups | grep focusflow"
            />
          </div>

          <Alert>
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertDescription>
              Now FocusFlow can modify /etc/hosts without requiring sudo, and you can manage access by adding/removing users from the focusflow group.
            </AlertDescription>
          </Alert>

          <CollapsibleSection title="How to revert these changes">
            <CodeBlock
              code={`# Restore original permissions
sudo chgrp root /etc/hosts
sudo chmod 644 /etc/hosts

# Remove the group
sudo groupdel focusflow

# Remove user from group (if group still exists)
sudo gpasswd -d $USER focusflow

# Restore from backup if needed
sudo cp /etc/hosts.backup /etc/hosts`}
            />
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Method 4: File Permissions (Not Recommended) */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-amber-500/10 text-amber-700 dark:text-amber-400">
              Not Recommended
            </Badge>
            <CardTitle className="text-lg">Method 4: Direct File Permissions</CardTitle>
          </div>
          <CardDescription>
            Make hosts file writable by your user - simple but least secure
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert variant="destructive">
            <ShieldAlert className="h-4 w-4" />
            <AlertDescription className="text-sm">
              This makes /etc/hosts writable by your user, meaning any application you run can modify it.
              Use Method 3 (group permissions) instead for better security.
            </AlertDescription>
          </Alert>

          <div className="space-y-3">
            <h4 className="font-medium flex items-center gap-2">
              <span className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                1
              </span>
              Backup and change ownership
            </h4>
            <CodeBlock
              code={`sudo cp /etc/hosts /etc/hosts.backup
sudo chown $USER:$USER /etc/hosts
sudo chmod 644 /etc/hosts`}
            />
          </div>

          <CollapsibleSection title="Revert to original permissions">
            <CodeBlock
              code={`sudo chown root:root /etc/hosts
sudo chmod 644 /etc/hosts
sudo cp /etc/hosts.backup /etc/hosts`}
            />
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Troubleshooting */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Terminal className="h-5 w-5" />
            Troubleshooting
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <CollapsibleSection title='"Permission denied" error'>
            <p className="text-sm text-muted-foreground mb-3">
              If you see "Permission denied" when trying to modify /etc/hosts:
            </p>
            <ul className="list-disc list-inside text-sm space-y-2 ml-4">
              <li>Verify you're using sudo for commands that require it</li>
              <li>Check your user is in the sudo group: <code className="text-xs bg-muted px-1 rounded">groups $USER | grep sudo</code></li>
              <li>If using group method, ensure you logged out and back in after adding yourself to the group</li>
              <li>Check file permissions: <code className="text-xs bg-muted px-1 rounded">ls -l /etc/hosts</code></li>
            </ul>
          </CollapsibleSection>

          <CollapsibleSection title="Websites still not blocked">
            <p className="text-sm text-muted-foreground mb-3">
              If blocking isn't working even after setup:
            </p>
            <ol className="list-decimal list-inside text-sm space-y-2 ml-4">
              <li>Flush DNS cache:
                <Tabs defaultValue="systemd-resolved" className="w-full mt-2">
                  <TabsList className="grid w-full grid-cols-3">
                    <TabsTrigger value="systemd-resolved">systemd</TabsTrigger>
                    <TabsTrigger value="nscd">nscd</TabsTrigger>
                    <TabsTrigger value="dnsmasq">dnsmasq</TabsTrigger>
                  </TabsList>
                  <TabsContent value="systemd-resolved">
                    <CodeBlock code="sudo systemd-resolve --flush-caches" />
                  </TabsContent>
                  <TabsContent value="nscd">
                    <CodeBlock code="sudo /etc/init.d/nscd restart" />
                  </TabsContent>
                  <TabsContent value="dnsmasq">
                    <CodeBlock code="sudo systemctl restart dnsmasq" />
                  </TabsContent>
                </Tabs>
              </li>
              <li className="mt-3">Verify entries were added:
                <CodeBlock code="cat /etc/hosts | grep focusflow" />
              </li>
              <li>Restart your browser completely</li>
              <li>Check if browser uses DNS-over-HTTPS and disable it in browser settings</li>
              <li>Some apps use their own DNS resolver - check app settings</li>
            </ol>
          </CollapsibleSection>

          <CollapsibleSection title="SELinux or AppArmor blocking access">
            <p className="text-sm text-muted-foreground mb-3">
              Security modules like SELinux (Fedora/RHEL) or AppArmor (Ubuntu) might interfere:
            </p>

            <h5 className="font-medium mt-3 mb-2">SELinux (Fedora/RHEL/CentOS):</h5>
            <CodeBlock
              code={`# Check if SELinux is blocking
sudo ausearch -m AVC -ts recent | grep focusflow

# If blocked, create a custom policy
sudo ausearch -m AVC -ts recent | audit2allow -M focusflow-hosts
sudo semodule -i focusflow-hosts.pp

# Or temporarily set to permissive (not recommended for production)
sudo setenforce 0`}
            />

            <h5 className="font-medium mt-3 mb-2">AppArmor (Ubuntu/Debian):</h5>
            <CodeBlock
              code={`# Check AppArmor status
sudo aa-status

# If FocusFlow has a profile, put it in complain mode
sudo aa-complain /path/to/focusflow

# View denials
sudo journalctl | grep -i apparmor | grep focusflow`}
            />
          </CollapsibleSection>

          <CollapsibleSection title="Polkit rule not working">
            <p className="text-sm text-muted-foreground mb-3">
              If the Polkit rule isn't taking effect:
            </p>
            <ul className="list-disc list-inside text-sm space-y-2 ml-4">
              <li>Check Polkit service is running: <code className="text-xs bg-muted px-1 rounded">systemctl status polkit</code></li>
              <li>Verify rule syntax:
                <CodeBlock code="pkaction --verbose" />
              </li>
              <li>Check Polkit logs:
                <CodeBlock code="journalctl -u polkit" />
              </li>
              <li>Ensure file has correct permissions (644) and ownership (root:root)</li>
            </ul>
          </CollapsibleSection>

          <CollapsibleSection title="Restore default hosts file">
            <p className="text-sm text-muted-foreground mb-3">
              To restore a clean hosts file:
            </p>
            <CodeBlock
              code={`# Restore from backup
sudo cp /etc/hosts.backup /etc/hosts

# Or create a fresh one
sudo tee /etc/hosts > /dev/null <<'EOF'
127.0.0.1   localhost
127.0.1.1   $(hostname)

# The following lines are desirable for IPv6 capable hosts
::1     localhost ip6-localhost ip6-loopback
ff02::1 ip6-allnodes
ff02::2 ip6-allrouters
EOF

# Flush DNS cache (method depends on your system)
sudo systemd-resolve --flush-caches  # systemd
# OR
sudo /etc/init.d/nscd restart        # nscd`}
            />
          </CollapsibleSection>
        </CardContent>
      </Card>

      {/* Distribution-Specific Notes */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Package className="h-5 w-5" />
            Distribution-Specific Notes
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="ubuntu" className="w-full">
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="ubuntu">Ubuntu/Debian</TabsTrigger>
              <TabsTrigger value="fedora">Fedora</TabsTrigger>
              <TabsTrigger value="arch">Arch</TabsTrigger>
              <TabsTrigger value="opensuse">openSUSE</TabsTrigger>
            </TabsList>

            <TabsContent value="ubuntu" className="space-y-3 mt-4">
              <p className="text-sm text-muted-foreground">
                Ubuntu and Debian use AppArmor by default. The group method (Method 3) works best here.
              </p>
              <CodeBlock
                code={`# Recommended for Ubuntu/Debian
sudo groupadd focusflow
sudo usermod -a -G focusflow $USER
sudo chgrp focusflow /etc/hosts
sudo chmod 664 /etc/hosts`}
              />
            </TabsContent>

            <TabsContent value="fedora" className="space-y-3 mt-4">
              <p className="text-sm text-muted-foreground">
                Fedora uses SELinux by default. You may need to adjust SELinux policies.
              </p>
              <CodeBlock
                code={`# For Fedora with SELinux
sudo setsebool -P allow_user_exec_content 1

# Or create custom policy (safer)
sudo ausearch -m AVC | audit2allow -M focusflow-custom
sudo semodule -i focusflow-custom.pp`}
              />
            </TabsContent>

            <TabsContent value="arch" className="space-y-3 mt-4">
              <p className="text-sm text-muted-foreground">
                Arch Linux typically doesn't use SELinux or AppArmor. Any method will work smoothly.
              </p>
              <CodeBlock
                code={`# Recommended for Arch
sudo groupadd focusflow
sudo usermod -a -G focusflow $USER
sudo chgrp focusflow /etc/hosts
sudo chmod 664 /etc/hosts

# Don't forget to log out and back in!`}
              />
            </TabsContent>

            <TabsContent value="opensuse" className="space-y-3 mt-4">
              <p className="text-sm text-muted-foreground">
                openSUSE uses AppArmor. The Polkit method works well here.
              </p>
              <CodeBlock
                code={`# For openSUSE
sudo zypper install polkit

# Then follow Method 2 (Polkit rule)`}
              />
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  );
}
