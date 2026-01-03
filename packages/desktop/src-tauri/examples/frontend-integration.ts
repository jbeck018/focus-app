/**
 * Frontend Integration Examples for Privilege Handling
 *
 * This file demonstrates how to integrate the privilege handling system
 * into your frontend application.
 */

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Type Definitions
// ============================================================================

interface BlockingCapabilities {
  hosts_file_writable: boolean;
  hosts_file_path: string;
  process_termination_available: boolean;
  recommended_method: 'hosts_file' | 'process_termination' | 'frontend_only';
  available_methods: Array<'hosts_file' | 'process_termination' | 'frontend_only'>;
  limitations: string[];
  platform: string;
}

interface ElevationInstructions {
  platform: string;
  primary_method: string;
  alternative_methods: string[];
  steps: string[];
  security_notes: string[];
  requires_restart: boolean;
}

// ============================================================================
// Example 1: Check Capabilities on App Initialization
// ============================================================================

async function initializeBlockingSystem(): Promise<void> {
  console.log('Checking blocking capabilities...');

  const capabilities = await invoke<BlockingCapabilities>('get_blocking_capabilities');

  console.log('Platform:', capabilities.platform);
  console.log('Hosts file writable:', capabilities.hosts_file_writable);
  console.log('Recommended method:', capabilities.recommended_method);

  if (!capabilities.hosts_file_writable) {
    console.warn('⚠ Website blocking requires elevated permissions');
    console.warn('Limitations:', capabilities.limitations);

    // Show notification to user
    showPermissionNotification(capabilities);
  } else {
    console.log('✓ Full website blocking available');
  }

  // Store capabilities in app state (React, Vue, Svelte, etc.)
  // setAppState({ blockingCapabilities: capabilities });
}

// ============================================================================
// Example 2: Display Permission Setup Instructions
// ============================================================================

async function showPermissionSetupGuide(): Promise<void> {
  const instructions = await invoke<ElevationInstructions>('get_elevation_instructions');

  console.log('='.repeat(60));
  console.log(`Setup Instructions for ${instructions.platform}`);
  console.log('='.repeat(60));
  console.log();
  console.log(`Method: ${instructions.primary_method}`);
  console.log();
  console.log('Steps:');
  instructions.steps.forEach((step, i) => {
    console.log(`${i + 1}. ${step}`);
  });
  console.log();

  if (instructions.alternative_methods.length > 0) {
    console.log('Alternative Methods:');
    instructions.alternative_methods.forEach((method) => {
      console.log(`  - ${method}`);
    });
    console.log();
  }

  console.log('Security Notes:');
  instructions.security_notes.forEach((note) => {
    console.log(`  - ${note}`);
  });
  console.log();

  if (instructions.requires_restart) {
    console.log('⚠ You will need to restart the app after granting permissions');
  }
  console.log('='.repeat(60));

  // Display in UI modal/dialog
  // showInstructionsModal(instructions);
}

// ============================================================================
// Example 3: Periodic Permission Polling
// ============================================================================

/**
 * Poll for permission changes every 5 seconds
 * Useful when waiting for user to grant permissions
 */
function startPermissionPolling(onPermissionGranted: () => void): () => void {
  const intervalId = setInterval(async () => {
    const hasPermission = await invoke<boolean>('check_hosts_file_permissions');

    if (hasPermission) {
      console.log('✓ Permissions granted!');
      onPermissionGranted();
      clearInterval(intervalId);
    }
  }, 5000);

  // Return cleanup function
  return () => clearInterval(intervalId);
}

// Usage:
// const stopPolling = startPermissionPolling(() => {
//   showSuccessNotification('Website blocking enabled!');
//   refreshBlockingCapabilities();
// });

// ============================================================================
// Example 4: React Component Integration
// ============================================================================

/**
 * React Hook for managing blocking capabilities
 */
function useBlockingCapabilities() {
  // In a real React app:
  // const [capabilities, setCapabilities] = useState<BlockingCapabilities | null>(null);
  // const [loading, setLoading] = useState(true);
  // const [error, setError] = useState<string | null>(null);

  // useEffect(() => {
  //   async function loadCapabilities() {
  //     try {
  //       const caps = await invoke<BlockingCapabilities>('get_blocking_capabilities');
  //       setCapabilities(caps);
  //     } catch (err) {
  //       setError(err.toString());
  //     } finally {
  //       setLoading(false);
  //     }
  //   }
  //
  //   loadCapabilities();
  // }, []);

  // return { capabilities, loading, error };
}

/**
 * React Component: Permission Banner
 */
function PermissionBanner() {
  // const { capabilities } = useBlockingCapabilities();
  // const [showInstructions, setShowInstructions] = useState(false);

  // if (!capabilities || capabilities.hosts_file_writable) {
  //   return null;
  // }

  // return (
  //   <div className="banner warning">
  //     <div className="banner-icon">⚠</div>
  //     <div className="banner-content">
  //       <h3>Website blocking requires elevated permissions</h3>
  //       <p>{capabilities.limitations.join(' ')}</p>
  //       <button onClick={() => setShowInstructions(true)}>
  //         Show Setup Instructions
  //       </button>
  //     </div>
  //     {showInstructions && <InstructionsModal />}
  //   </div>
  // );
}

/**
 * React Component: Instructions Modal
 */
function InstructionsModal() {
  // const [instructions, setInstructions] = useState<ElevationInstructions | null>(null);
  // const [loading, setLoading] = useState(true);

  // useEffect(() => {
  //   async function loadInstructions() {
  //     try {
  //       const inst = await invoke<ElevationInstructions>('get_elevation_instructions');
  //       setInstructions(inst);
  //     } finally {
  //       setLoading(false);
  //     }
  //   }
  //
  //   loadInstructions();
  // }, []);

  // if (loading || !instructions) {
  //   return <div>Loading...</div>;
  // }

  // return (
  //   <div className="modal">
  //     <h2>Setup Website Blocking for {instructions.platform}</h2>
  //
  //     <section>
  //       <h3>Method: {instructions.primary_method}</h3>
  //       <ol>
  //         {instructions.steps.map((step, i) => (
  //           <li key={i}>{step}</li>
  //         ))}
  //       </ol>
  //     </section>
  //
  //     {instructions.alternative_methods.length > 0 && (
  //       <section>
  //         <h3>Alternative Methods</h3>
  //         <ul>
  //           {instructions.alternative_methods.map((method, i) => (
  //             <li key={i}>{method}</li>
  //           ))}
  //         </ul>
  //       </section>
  //     )}
  //
  //     <section>
  //       <h3>Security Notes</h3>
  //       <ul>
  //         {instructions.security_notes.map((note, i) => (
  //           <li key={i}>{note}</li>
  //         ))}
  //       </ul>
  //     </section>
  //
  //     {instructions.requires_restart && (
  //       <div className="alert info">
  //         You'll need to restart FocusFlow after granting permissions
  //       </div>
  //     )}
  //   </div>
  // );
}

/**
 * React Component: Blocking Status Indicator
 */
function BlockingStatus() {
  // const { capabilities } = useBlockingCapabilities();

  // if (!capabilities) {
  //   return null;
  // }

  // const statusConfig = {
  //   hosts_file: {
  //     label: 'Full Blocking',
  //     description: 'System-wide blocking via hosts file',
  //     variant: 'success',
  //   },
  //   process_termination: {
  //     label: 'App Blocking',
  //     description: 'Application blocking via process termination',
  //     variant: 'warning',
  //   },
  //   frontend_only: {
  //     label: 'Limited Blocking',
  //     description: 'Frontend-based blocking (can be bypassed)',
  //     variant: 'error',
  //   },
  // };

  // const status = statusConfig[capabilities.recommended_method];

  // return (
  //   <div className={`status-badge ${status.variant}`}>
  //     <div className="status-label">{status.label}</div>
  //     <div className="status-description">{status.description}</div>
  //   </div>
  // );
}

// ============================================================================
// Example 5: Complete Setup Flow
// ============================================================================

async function completeSetupFlow(): Promise<void> {
  console.log('Starting blocking setup flow...');

  // Step 1: Check current capabilities
  const capabilities = await invoke<BlockingCapabilities>('get_blocking_capabilities');

  console.log('Current status:');
  console.log('  Platform:', capabilities.platform);
  console.log('  Hosts file writable:', capabilities.hosts_file_writable);
  console.log('  Recommended method:', capabilities.recommended_method);

  if (capabilities.hosts_file_writable) {
    console.log('✓ Setup complete - full blocking available');
    return;
  }

  // Step 2: Show what's needed
  console.log('\n⚠ Setup required for full blocking');
  console.log('Limitations:');
  capabilities.limitations.forEach((limitation) => {
    console.log('  -', limitation);
  });

  // Step 3: Get and display instructions
  console.log('\nFetching setup instructions...');
  await showPermissionSetupGuide();

  // Step 4: Wait for user to grant permissions
  console.log('\nWaiting for permissions...');
  console.log('Please follow the instructions above and then restart the app.');

  // In a real app, you'd start polling here:
  // const stopPolling = startPermissionPolling(() => {
  //   console.log('✓ Permissions granted and detected!');
  // });
}

// ============================================================================
// Example 6: Graceful Degradation
// ============================================================================

async function startBlockingWithGracefulDegradation(domains: string[]): Promise<void> {
  // Check what's available
  const hasPermission = await invoke<boolean>('check_hosts_file_permissions');

  if (hasPermission) {
    // Use hosts file (most secure)
    console.log('Using hosts file blocking (most secure)');
    // Call your existing toggle_blocking command
    await invoke('toggle_blocking', { enable: true });
  } else {
    // Use fallback methods
    console.log('Using fallback blocking methods');
    console.log('⚠ Warning: This can be bypassed more easily');

    // Still enable blocking (uses in-memory state + DNS fallback)
    await invoke('toggle_blocking', { enable: true });

    // Show notification to user
    console.warn('Website blocking is active but may be less effective.');
    console.warn('For best results, grant elevated permissions.');

    // Optional: Show setup instructions
    // showPermissionSetupGuide();
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

function showPermissionNotification(capabilities: BlockingCapabilities): void {
  // In a real app, use your notification system
  console.warn('='.repeat(60));
  console.warn('PERMISSION REQUIRED');
  console.warn('='.repeat(60));
  console.warn('Website blocking requires elevated permissions.');
  console.warn('Current limitations:');
  capabilities.limitations.forEach((limitation) => {
    console.warn('  -', limitation);
  });
  console.warn('Click "Setup Instructions" to enable full blocking.');
  console.warn('='.repeat(60));
}

// ============================================================================
// Export for use in your app
// ============================================================================

export {
  // Type definitions
  type BlockingCapabilities,
  type ElevationInstructions,

  // Initialization
  initializeBlockingSystem,

  // Instructions
  showPermissionSetupGuide,

  // Polling
  startPermissionPolling,

  // React components (commented out - uncomment in real app)
  // useBlockingCapabilities,
  // PermissionBanner,
  // InstructionsModal,
  // BlockingStatus,

  // Complete flows
  completeSetupFlow,
  startBlockingWithGracefulDegradation,

  // Helpers
  showPermissionNotification,
};

// ============================================================================
// Example Usage in Main App
// ============================================================================

/**
 * Example: Initialize on app startup
 */
async function main() {
  // Initialize blocking system and check capabilities
  await initializeBlockingSystem();

  // If you want to show the full setup flow:
  // await completeSetupFlow();
}

// Run example
// main().catch(console.error);
