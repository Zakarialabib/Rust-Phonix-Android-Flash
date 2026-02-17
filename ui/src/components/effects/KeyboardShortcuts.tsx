import { createEffect, onCleanup } from 'solid-js';
import { createShortcut } from '@solid-primitives/keyboard';
import { globalStore, setGlobalStore } from '../../store';

export type ShortcutHandler = (e: KeyboardEvent) => void;

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean;
  handler: ShortcutHandler;
  description: string;
}

// Hook for registering a single keyboard shortcut
export function useShortcut(
  keys: string[],
  callback: (e: KeyboardEvent | null) => void
) {
  createShortcut(keys, callback);
}

// Global keyboard shortcuts for the app
export function useGlobalShortcuts() {
  // Navigation shortcuts
  createShortcut(['Control', '1'], () => setGlobalStore('activeTab', 'detect'));
  createShortcut(['Control', '2'], () => setGlobalStore('activeTab', 'diagnostics'));
  createShortcut(['Control', '3'], () => setGlobalStore('activeTab', 'config'));
  createShortcut(['Control', '4'], () => setGlobalStore('activeTab', 'check'));
  createShortcut(['Control', '5'], () => setGlobalStore('activeTab', 'build'));
  createShortcut(['Control', '6'], () => setGlobalStore('activeTab', 'flash'));
  
  // Toggle sidebar
  createShortcut(['Control', 'b'], () => 
    setGlobalStore('sidebarCollapsed', !globalStore.sidebarCollapsed)
  );
  
  // Command palette (placeholder)
  createShortcut(['Control', 'k'], () => {});
  
  // Reload
  createShortcut(['Control', 'Shift', 'r'], () => window.location.reload());
}

// Get list of all shortcuts for help display
export function getShortcutsList(): KeyboardShortcut[] {
  return [
    {
      key: '1',
      ctrl: true,
      handler: () => {},
      description: 'Navigate to Discovery',
    },
    {
      key: '2',
      ctrl: true,
      handler: () => {},
      description: 'Navigate to Forensics',
    },
    {
      key: '3',
      ctrl: true,
      handler: () => {},
      description: 'Navigate to Architect',
    },
    {
      key: '4',
      ctrl: true,
      handler: () => {},
      description: 'Navigate to Compatibility',
    },
    {
      key: '5',
      ctrl: true,
      handler: () => {},
      description: 'Navigate to Foundry',
    },
    {
      key: '6',
      ctrl: true,
      handler: () => {},
      description: 'Navigate to Burner',
    },
    {
      key: 'b',
      ctrl: true,
      handler: () => {},
      description: 'Toggle Sidebar',
    },
    {
      key: 'k',
      ctrl: true,
      handler: () => {},
      description: 'Open Command Palette',
    },
    {
      key: 'r',
      ctrl: true,
      shift: true,
      handler: () => {},
      description: 'Reload Application',
    },
  ];
}

// Component to display keyboard shortcuts help
export function KeyboardShortcutsHelp() {
  const shortcuts = getShortcutsList();

  return (
    <div class="space-y-2">
      {shortcuts.map((shortcut) => (
        <div class="flex justify-between items-center text-xs">
          <span class="text-text-muted">{shortcut.description}</span>
          <kbd class="px-2 py-1 bg-sidebar border border-border-subtle rounded text-text-secondary font-mono">
            {shortcut.ctrl && 'Ctrl+'}{shortcut.shift && 'Shift+'}{shortcut.alt && 'Alt+'}{shortcut.key.toUpperCase()}
          </kbd>
        </div>
      ))}
    </div>
  );
}
