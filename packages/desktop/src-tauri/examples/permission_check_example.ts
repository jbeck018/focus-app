/**
 * Permission Check Example
 *
 * This file demonstrates how to use the FocusFlow permission checking system
 * in a frontend application. Copy and adapt these examples to your needs.
 */

// Import the type definitions
// In a real app: import { Commands, PermissionHelpers, PermissionStatus } from '../bindings/permissions';

// Mock Tauri invoke for this example
const invoke = (window as any).__TAURI__?.core?.invoke || (() => Promise.reject('Tauri not available'));

// ============================================================================
// Example 1: Simple Permission Check
// ============================================================================

async function simplePermissionCheck() {
  try {
    const status = await invoke('check_permissions');

    console.log('=== Permission Status ===');
    console.log('Platform:', status.platform);
    console.log('Overall Status:', status.overall_status);
    console.log('Hosts File Writable:', status.hosts_file_writable);
    console.log('Process Monitoring:', status.process_monitoring_available);
    console.log('Process Termination:', status.process_termination_available);

    if (status.overall_status !== 'fully_functional') {
      console.log('\n⚠️ Recommendations:');
      status.recommendations.forEach((rec: string, i: number) => {
        console.log(`  ${i + 1}. ${rec}`);
      });
    }
  } catch (error) {
    console.error('Failed to check permissions:', error);
  }
}

// ============================================================================
// Example 2: Get Setup Instructions
// ============================================================================

async function getSetupInstructions() {
  try {
    // Pass empty string to auto-detect platform
    const instructions = await invoke('get_permission_instructions', { platform: '' });

    console.log('\n=== Setup Instructions for', instructions.platform, '===\n');

    // Primary method (recommended)
    console.log('📌 RECOMMENDED METHOD:', instructions.primary_method.name);
    console.log('Type:', instructions.primary_method.is_permanent ? 'Permanent' : 'Temporary');
    console.log('\nSteps:');
    instructions.primary_method.steps.forEach((step: string, i: number) => {
      console.log(`  ${i + 1}. ${step}`);
    });
    console.log('\nGrants:');
    instructions.primary_method.grants.forEach((grant: string) => {
      console.log(`  - ${grant}`);
    });

    // Alternative methods
    if (instructions.alternative_methods.length > 0) {
      console.log('\n🔄 ALTERNATIVE METHODS:');
      instructions.alternative_methods.forEach((method: any, i: number) => {
        console.log(`\n${i + 1}. ${method.name} (${method.is_permanent ? 'Permanent' : 'Temporary'})`);
        method.steps.forEach((step: string, j: number) => {
          console.log(`   ${j + 1}. ${step}`);
        });
      });
    }

    // Security notes
    console.log('\n🔒 SECURITY NOTES:');
    instructions.security_notes.forEach((note: string) => {
      console.log(`  - ${note}`);
    });

    if (instructions.requires_restart) {
      console.log('\n⚠️ App restart required after granting permissions');
    }
  } catch (error) {
    console.error('Failed to get instructions:', error);
  }
}

// ============================================================================
// Example 3: React Component for Permission Setup
// ============================================================================

const PermissionSetupComponent = `
import React, { useState, useEffect } from 'react';
import { Commands } from '../bindings/permissions';
import type { PermissionStatus, PlatformInstructions } from '../bindings/permissions';

export function PermissionSetup() {
  const [status, setStatus] = useState<PermissionStatus | null>(null);
  const [instructions, setInstructions] = useState<PlatformInstructions | null>(null);
  const [loading, setLoading] = useState(true);
  const [checking, setChecking] = useState(false);

  useEffect(() => {
    checkPermissions();
  }, []);

  async function checkPermissions() {
    setLoading(true);
    try {
      const [permStatus, permInstructions] = await Promise.all([
        Commands.checkPermissions(),
        Commands.getPermissionInstructions('')
      ]);
      setStatus(permStatus);
      setInstructions(permInstructions);
    } catch (error) {
      console.error('Permission check failed:', error);
    } finally {
      setLoading(false);
    }
  }

  async function recheckPermissions() {
    setChecking(true);
    await checkPermissions();
    setChecking(false);
  }

  if (loading) {
    return <div className="loading">Checking permissions...</div>;
  }

  if (!status || !instructions) {
    return <div className="error">Failed to load permission status</div>;
  }

  // All good - no setup needed
  if (status.overall_status === 'fully_functional') {
    return (
      <div className="permission-status success">
        <h3>✅ All permissions granted</h3>
        <p>All blocking features are fully functional.</p>
      </div>
    );
  }

  // Needs setup
  return (
    <div className="permission-setup">
      <div className="status-header">
        <h2>Setup Required</h2>
        <StatusBadge status={status.overall_status} />
      </div>

      <div className="status-details">
        <h3>Current Status:</h3>
        <ul className="permission-list">
          <PermissionItem
            name="Website Blocking (Hosts File)"
            available={status.hosts_file_writable}
            error={status.hosts_file_error}
          />
          <PermissionItem
            name="App Monitoring"
            available={status.process_monitoring_available}
            error={status.process_monitoring_error}
          />
          <PermissionItem
            name="App Termination"
            available={status.process_termination_available}
            error={status.process_termination_error}
          />
        </ul>
      </div>

      <div className="recommendations">
        <h3>What to do:</h3>
        <ul>
          {status.recommendations.map((rec, i) => (
            <li key={i}>{rec}</li>
          ))}
        </ul>
      </div>

      <div className="instructions">
        <h3>Setup Instructions for {instructions.platform}</h3>

        <div className="method primary">
          <div className="method-header">
            <h4>{instructions.primary_method.name}</h4>
            <span className="badge">
              {instructions.primary_method.is_permanent ? '✓ Permanent' : '⚠ Temporary'}
            </span>
          </div>

          <ol className="steps">
            {instructions.primary_method.steps.map((step, i) => (
              <li key={i}>{step}</li>
            ))}
          </ol>

          <div className="grants">
            <strong>This grants:</strong>
            <ul>
              {instructions.primary_method.grants.map((grant, i) => (
                <li key={i}>{grant}</li>
              ))}
            </ul>
          </div>
        </div>

        {instructions.alternative_methods.length > 0 && (
          <details className="alternatives">
            <summary>Show alternative methods ({instructions.alternative_methods.length})</summary>
            {instructions.alternative_methods.map((method, i) => (
              <div key={i} className="method alternative">
                <h4>{method.name}</h4>
                <ol className="steps">
                  {method.steps.map((step, j) => (
                    <li key={j}>{step}</li>
                  ))}
                </ol>
              </div>
            ))}
          </details>
        )}

        <div className="security-notes">
          <h4>🔒 Security Information:</h4>
          <ul>
            {instructions.security_notes.map((note, i) => (
              <li key={i}>{note}</li>
            ))}
          </ul>
        </div>

        {instructions.requires_restart && (
          <div className="warning-box">
            ⚠️ You must restart FocusFlow after granting permissions for changes to take effect.
          </div>
        )}
      </div>

      <div className="actions">
        <button
          onClick={recheckPermissions}
          disabled={checking}
          className="btn-primary"
        >
          {checking ? 'Checking...' : 'Recheck Permissions'}
        </button>
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const statusConfig = {
    fully_functional: { label: 'Fully Functional', color: 'green' },
    degraded: { label: 'Degraded', color: 'yellow' },
    non_functional: { label: 'Non-Functional', color: 'red' }
  };

  const config = statusConfig[status as keyof typeof statusConfig];

  return (
    <span className={\`status-badge \${config.color}\`}>
      {config.label}
    </span>
  );
}

function PermissionItem({
  name,
  available,
  error
}: {
  name: string;
  available: boolean;
  error: string | null;
}) {
  return (
    <li className={\`permission-item \${available ? 'available' : 'unavailable'}\`}>
      <span className="icon">{available ? '✅' : '❌'}</span>
      <span className="name">{name}</span>
      {!available && error && (
        <span className="error">{error}</span>
      )}
    </li>
  );
}
`;

console.log('React Component Example:');
console.log(PermissionSetupComponent);

// ============================================================================
// Example 4: Svelte Component
// ============================================================================

const SvelteComponent = `
<script lang="ts">
  import { onMount } from 'svelte';
  import { Commands } from '../bindings/permissions';
  import type { PermissionStatus, PlatformInstructions } from '../bindings/permissions';

  let status: PermissionStatus | null = null;
  let instructions: PlatformInstructions | null = null;
  let loading = true;
  let checking = false;

  onMount(async () => {
    await checkPermissions();
  });

  async function checkPermissions() {
    loading = true;
    try {
      [status, instructions] = await Promise.all([
        Commands.checkPermissions(),
        Commands.getPermissionInstructions('')
      ]);
    } catch (error) {
      console.error('Permission check failed:', error);
    } finally {
      loading = false;
    }
  }

  async function recheckPermissions() {
    checking = true;
    await checkPermissions();
    checking = false;
  }
</script>

{#if loading}
  <div class="loading">Checking permissions...</div>
{:else if status && instructions}
  {#if status.overall_status === 'fully_functional'}
    <div class="success">
      <h3>✅ All permissions granted</h3>
    </div>
  {:else}
    <div class="permission-setup">
      <h2>Setup Required</h2>
      <p>Status: {status.overall_status}</p>

      <div class="instructions">
        <h3>{instructions.primary_method.name}</h3>
        <ol>
          {#each instructions.primary_method.steps as step}
            <li>{step}</li>
          {/each}
        </ol>
      </div>

      <button on:click={recheckPermissions} disabled={checking}>
        {checking ? 'Checking...' : 'Recheck Permissions'}
      </button>
    </div>
  {/if}
{:else}
  <div class="error">Failed to load permissions</div>
{/if}
`;

console.log('\n\nSvelte Component Example:');
console.log(SvelteComponent);

// ============================================================================
// Example 5: Periodic Permission Monitoring
// ============================================================================

class PermissionMonitor {
  private intervalId: number | null = null;
  private listeners: Array<(status: any) => void> = [];
  private currentStatus: any = null;

  start(intervalMs: number = 30000) {
    console.log('Starting permission monitor...');

    // Check immediately
    this.checkAndNotify();

    // Then check periodically
    this.intervalId = window.setInterval(() => {
      this.checkAndNotify();
    }, intervalMs);
  }

  stop() {
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
      this.intervalId = null;
      console.log('Stopped permission monitor');
    }
  }

  subscribe(callback: (status: any) => void) {
    this.listeners.push(callback);
    // Immediately notify with current status if available
    if (this.currentStatus) {
      callback(this.currentStatus);
    }
    return () => {
      this.listeners = this.listeners.filter(l => l !== callback);
    };
  }

  private async checkAndNotify() {
    try {
      const status = await invoke('check_permissions');

      // Detect status changes
      if (this.currentStatus &&
          this.currentStatus.overall_status !== status.overall_status) {
        console.log('⚠️ Permission status changed:',
          this.currentStatus.overall_status, '→', status.overall_status);
      }

      this.currentStatus = status;
      this.listeners.forEach(listener => listener(status));
    } catch (error) {
      console.error('Permission check failed:', error);
    }
  }
}

// Usage:
const monitor = new PermissionMonitor();
monitor.subscribe((status) => {
  console.log('Permission status update:', status.overall_status);
});
monitor.start(30000); // Check every 30 seconds

// ============================================================================
// Example 6: CLI-style Interactive Permission Checker
// ============================================================================

async function interactivePermissionChecker() {
  console.log('\n╔════════════════════════════════════════╗');
  console.log('║   FocusFlow Permission Checker         ║');
  console.log('╚════════════════════════════════════════╝\n');

  const status = await invoke('check_permissions');

  console.log('Platform:', status.platform);
  console.log('Hosts File Path:', status.hosts_file_path);
  console.log('\n' + '─'.repeat(40) + '\n');

  // Check each permission
  console.log('PERMISSION STATUS:\n');

  console.log('1. Hosts File Access');
  console.log('   Status:', status.hosts_file_writable ? '✅ GRANTED' : '❌ DENIED');
  if (status.hosts_file_error) {
    console.log('   Error:', status.hosts_file_error);
  }

  console.log('\n2. Process Monitoring');
  console.log('   Status:', status.process_monitoring_available ? '✅ AVAILABLE' : '❌ UNAVAILABLE');
  if (status.process_monitoring_error) {
    console.log('   Error:', status.process_monitoring_error);
  }

  console.log('\n3. Process Termination');
  console.log('   Status:', status.process_termination_available ? '✅ AVAILABLE' : '❌ UNAVAILABLE');
  if (status.process_termination_error) {
    console.log('   Error:', status.process_termination_error);
  }

  console.log('\n' + '─'.repeat(40) + '\n');

  // Overall status
  const statusSymbols = {
    fully_functional: '✅',
    degraded: '⚠️',
    non_functional: '❌'
  };

  console.log('OVERALL STATUS:',
    statusSymbols[status.overall_status as keyof typeof statusSymbols],
    status.overall_status.toUpperCase().replace('_', ' '));

  // Recommendations
  if (status.recommendations.length > 0) {
    console.log('\n' + '─'.repeat(40) + '\n');
    console.log('RECOMMENDATIONS:\n');
    status.recommendations.forEach((rec: string, i: number) => {
      console.log(`${i + 1}. ${rec}`);
    });
  }

  // Offer to show instructions
  if (status.overall_status !== 'fully_functional') {
    console.log('\n' + '─'.repeat(40) + '\n');
    console.log('💡 Tip: Call getSetupInstructions() to see detailed setup steps');
  }

  console.log('\n');
}

// ============================================================================
// Run Examples
// ============================================================================

export async function runAllExamples() {
  console.log('Running permission check examples...\n');

  await interactivePermissionChecker();
  await simplePermissionCheck();
  await getSetupInstructions();

  console.log('\n✅ All examples completed!');
}

// Export for use in other modules
export {
  simplePermissionCheck,
  getSetupInstructions,
  interactivePermissionChecker,
  PermissionMonitor
};
