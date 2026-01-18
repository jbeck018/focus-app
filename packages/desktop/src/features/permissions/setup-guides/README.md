# Permissions Setup Guides

Comprehensive, platform-specific setup guides that help users configure permissions for FocusFlow's DNS-level blocking features.

## Overview

FocusFlow requires write access to the system's hosts file (`/etc/hosts` on macOS/Linux, `C:\Windows\System32\drivers\etc\hosts` on Windows) to implement DNS-level blocking. These guides provide step-by-step instructions for each platform with:

- Multiple setup methods with clear security trade-offs
- Copyable code blocks with one-click copy functionality
- Collapsible sections for advanced options
- Comprehensive troubleshooting sections
- Platform detection and automatic guide selection

## Components

### Main Components

#### `SetupGuides`
The main component that automatically detects the platform and displays the appropriate guide.

```tsx
<SetupGuides 
  showHeader={true}
  onComplete={() => console.log("Setup complete")} 
/>
```

#### `SetupGuidesModal`
Dialog wrapper for the setup guides, perfect for settings pages.

```tsx
<SetupGuidesModal
  open={open}
  onOpenChange={setOpen}
  onComplete={() => setOpen(false)}
/>
```

#### `SetupGuidesButton`
Pre-built button that opens the setup modal - drop-in solution.

```tsx
<SetupGuidesButton onComplete={() => console.log("Done!")}>
  Setup Permissions
</SetupGuidesButton>
```

### Platform-Specific Guides

#### `MacOSGuide`
macOS-specific instructions with three methods:
1. **Temporary Sudo** (Recommended) - Most secure, requires password each launch
2. **NOPASSWD Sudoers** - Convenient, passwordless access to hosts file
3. **File Permissions** - Not recommended, makes hosts file user-writable

#### `WindowsGuide`
Windows-specific instructions with three methods:
1. **Run as Administrator** (Recommended) - UAC prompt each launch
2. **Admin Shortcut** - Automatic elevation via shortcut properties
3. **File Permissions** - Grant user account specific permissions

#### `LinuxGuide`
Linux-specific instructions with four methods:
1. **Sudo** (Recommended) - Password required each launch
2. **Polkit Rule** - Modern authorization framework, no password
3. **Group Permissions** - Create dedicated group with access
4. **File Permissions** - Not recommended, user-writable hosts

Includes distribution-specific notes for:
- Ubuntu/Debian
- Fedora/RHEL
- Arch Linux
- openSUSE

### Utilities

#### `usePlatform` Hook
Custom hook for platform detection:

```tsx
const { platform, isLoading } = usePlatform();
// platform: "macos" | "windows" | "linux" | null
```

## Features

### Interactive Elements

- **Copy Buttons**: All code blocks have one-click copy functionality
- **Collapsible Sections**: Advanced options and troubleshooting hidden by default
- **Tab Navigation**: Switch between platform guides manually
- **Platform Detection**: Automatically shows guide for current platform
- **Platform Indicators**: Visual badges show current platform

### Content Structure

Each platform guide includes:

1. **Overview Alert**: Explains what permissions are needed and why
2. **Security Warning**: Clear explanation of security implications
3. **Setup Methods**: Multiple approaches with pros/cons
   - Step-by-step numbered instructions
   - Copyable terminal commands
   - Expected output examples
   - Security warnings where applicable
4. **Troubleshooting**: Comprehensive solutions for common issues
   - Permission denied errors
   - Websites not being blocked
   - Platform-specific security modules (SELinux, AppArmor, UAC)
   - DNS cache flushing
   - Browser DNS-over-HTTPS settings
   - How to restore defaults

### Code Blocks

All code blocks feature:
- Syntax highlighting with monospace font
- Copy button (appears on hover)
- Language indicator (PowerShell, Bash, etc.)
- Description text explaining what the command does
- Toast notification on successful copy

### Accessibility

- Proper ARIA labels on interactive elements
- Keyboard navigation support
- Screen reader friendly
- Semantic HTML structure
- Focus indicators on all interactive elements
- Collapsible sections with proper aria-expanded

## Usage Examples

### Example 1: Onboarding Flow

```tsx
import { SetupGuides } from "@/features/permissions";

function OnboardingStep() {
  const { nextStep } = useOnboarding();
  
  return (
    <SetupGuides 
      showHeader={true}
      onComplete={nextStep}
    />
  );
}
```

### Example 2: Settings Page

```tsx
import { SetupGuidesButton } from "@/features/permissions";

function SettingsPage() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Blocking Permissions</CardTitle>
      </CardHeader>
      <CardContent>
        <SetupGuidesButton variant="outline">
          View Setup Instructions
        </SetupGuidesButton>
      </CardContent>
    </Card>
  );
}
```

### Example 3: Permission Check Integration

```tsx
import { SetupGuidesModal, usePlatform } from "@/features/permissions";

function BlockingFeature() {
  const [showSetup, setShowSetup] = useState(false);
  const { platform } = usePlatform();
  
  const checkAndStartBlocking = async () => {
    try {
      await invoke("start_blocking");
    } catch (error) {
      if (error.includes("Permission denied")) {
        setShowSetup(true);
      }
    }
  };
  
  return (
    <>
      <Button onClick={checkAndStartBlocking}>Start Blocking</Button>
      <SetupGuidesModal 
        open={showSetup}
        onOpenChange={setShowSetup}
        defaultPlatform={platform}
      />
    </>
  );
}
```

### Example 4: First-Time Setup

```tsx
import { SetupGuides } from "@/features/permissions";

function FirstTimeSetup() {
  const handleComplete = () => {
    localStorage.setItem("hasCompletedSetup", "true");
    navigate("/dashboard");
  };
  
  return (
    <div className="container max-w-5xl mx-auto py-8">
      <SetupGuides 
        showHeader={true}
        onComplete={handleComplete}
      />
    </div>
  );
}
```

See `USAGE_EXAMPLE.tsx` for 10+ complete examples.

## API Reference

### `SetupGuides` Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `showHeader` | `boolean` | `true` | Show the introductory header card |
| `defaultPlatform` | `Platform` | auto-detected | Override platform detection |
| `onComplete` | `() => void` | `undefined` | Callback when user clicks Continue |

### `SetupGuidesModal` Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `open` | `boolean` | Yes | Control dialog visibility |
| `onOpenChange` | `(open: boolean) => void` | Yes | Handle dialog state changes |
| `onComplete` | `() => void` | No | Callback when setup is complete |
| `defaultPlatform` | `Platform` | No | Override platform detection |
| `title` | `string` | `"Setup Blocking Permissions"` | Custom dialog title |
| `description` | `string` | `"Follow these steps..."` | Custom dialog description |

### `SetupGuidesButton` Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `"default" \| "outline" \| "ghost"` | `"outline"` | Button style variant |
| `size` | `"default" \| "sm" \| "lg"` | `"default"` | Button size |
| `defaultPlatform` | `Platform` | auto-detected | Override platform detection |
| `onComplete` | `() => void` | `undefined` | Callback when setup is complete |
| `children` | `React.ReactNode` | `"Setup Permissions"` | Button text |

### `usePlatform` Returns

| Property | Type | Description |
|----------|------|-------------|
| `platform` | `Platform \| null` | Detected platform or null if detection failed |
| `isLoading` | `boolean` | Loading state during platform detection |

### `Platform` Type

```typescript
type Platform = "macos" | "windows" | "linux";
```

## File Structure

```
setup-guides/
├── index.tsx              # Main component with platform detection
├── macos-guide.tsx        # macOS-specific instructions
├── windows-guide.tsx      # Windows-specific instructions
├── linux-guide.tsx        # Linux-specific instructions
├── USAGE_EXAMPLE.tsx      # 10+ usage examples
└── README.md             # This file
```

## Customization

### Override Platform Detection

```tsx
<SetupGuides defaultPlatform="windows" />
```

### Hide Header Section

```tsx
<SetupGuides showHeader={false} />
```

### Custom Modal Title

```tsx
<SetupGuidesModal
  title="Fix Permission Error"
  description="Complete these steps to enable blocking"
  open={open}
  onOpenChange={setOpen}
/>
```

### Use Individual Platform Guides

```tsx
import { MacOSGuide } from "@/features/permissions";

<MacOSGuide />
```

## Platform Detection

The component uses Tauri's `@tauri-apps/plugin-os` to detect the platform:

```typescript
import { platform } from "@tauri-apps/plugin-os";

const platformName = await platform();
// Returns: "macos" | "windows" | "linux" | "ios" | "android"
```

The detection logic:
- `macos` and `ios` → Show macOS guide
- `windows` → Show Windows guide
- `linux` and `android` → Show Linux guide

## Security Considerations

### macOS
- **Recommended**: Temporary sudo (most secure)
- **Convenient**: NOPASSWD sudoers (passwordless but controlled)
- **Not Recommended**: File permissions (any process can modify)

### Windows
- **Recommended**: Run as Administrator (UAC each launch)
- **Convenient**: Admin shortcut (automatic elevation)
- **Advanced**: File permissions (controlled access)

### Linux
- **Recommended**: Sudo (password required)
- **Convenient**: Polkit rule (modern authorization)
- **Flexible**: Group permissions (granular control)
- **Not Recommended**: File permissions (user-writable)

## Troubleshooting Guide Coverage

Each platform guide includes solutions for:

### Common Issues
- Permission denied errors
- Websites not being blocked
- DNS cache not flushing
- Browser DNS-over-HTTPS bypassing blocks

### Platform-Specific
- **macOS**: Keychain access, SIP conflicts
- **Windows**: UAC prompts, antivirus interference
- **Linux**: SELinux/AppArmor, systemd-resolved, nscd

### Recovery
- How to restore original hosts file
- How to revert permission changes
- How to flush DNS cache
- How to check if changes took effect

## Best Practices

1. **Show at First Use**: Display guides when users first try to enable blocking
2. **Detect Permissions**: Check if permissions are already set before showing
3. **Provide Context**: Explain why permissions are needed
4. **Make Accessible**: Always provide a way to access from settings
5. **Handle Errors**: Show setup guide when permission errors occur
6. **Test Changes**: Provide way to verify setup worked

## Design Patterns

### Component Composition
```tsx
<SetupGuides>
  <PlatformDetection />
  <TabNavigation>
    <MacOSGuide />
    <WindowsGuide />
    <LinuxGuide />
  </TabNavigation>
  <ContinueButton />
</SetupGuides>
```

### Progressive Disclosure
- Main methods shown by default
- Advanced options in collapsible sections
- Troubleshooting hidden until needed
- Distribution-specific details in tabs

### Copy-First Design
- All commands have copy buttons
- Toast feedback on copy
- No manual typing required
- Reduces user errors

## Styling

Uses Tailwind CSS with shadcn/ui components:
- `Card` for section containers
- `Alert` for warnings and info
- `Tabs` for platform switching
- `Badge` for status indicators
- `Button` for interactive elements

All styles are responsive and work on mobile devices.

## Contributing

When adding new methods or troubleshooting:

1. **Add to appropriate platform guide** (macos-guide.tsx, windows-guide.tsx, or linux-guide.tsx)
2. **Use consistent structure**:
   - Badge for method type (Recommended, Convenient, etc.)
   - Numbered steps
   - Code blocks with descriptions
   - Security warnings where applicable
3. **Add to collapsible sections** for advanced content
4. **Update troubleshooting** if it addresses common issues
5. **Test on actual platform** to verify commands work

## Future Enhancements

Potential improvements:
- [ ] Video tutorials for each method
- [ ] Automated permission checking
- [ ] One-click setup scripts
- [ ] Platform-specific icons
- [ ] Progress indicators for multi-step processes
- [ ] Integration with native permission APIs
- [ ] Automatic fallback to degraded mode
- [ ] Analytics on which methods users choose

## Dependencies

- `@tauri-apps/plugin-os` - Platform detection
- `lucide-react` - Icons
- `sonner` - Toast notifications
- `@/components/ui/*` - shadcn/ui components

## License

Part of the FocusFlow application.
