/**
 * "选择一个已连接的终端" (v3.1 doc §1/§2/§3): lists open terminal tabs that
 * are actually bound to a connected remote device, and lets the user pick
 * one. Same `Menu` convention as the directory tree's row context menu
 * (`directoryTreePanel.ts`).
 */

import type { App, WorkspaceLeaf } from 'obsidian';
import { Menu, Notice } from 'obsidian';

import type TerminalPlugin from '../../main';
import { t } from '../../i18n';
import { TERMINAL_VIEW_TYPE, TerminalView } from './terminalView';

export interface ConnectedTerminal {
  leaf: WorkspaceLeaf;
  view: TerminalView;
  nodeId: string;
  label: string;
}

/** Open terminal tabs whose device connection is currently live. */
export function listConnectedTerminals(app: App, plugin: TerminalPlugin): ConnectedTerminal[] {
  const connections = plugin.getDeviceConnectionManager();
  const terminals: ConnectedTerminal[] = [];

  for (const leaf of app.workspace.getLeavesOfType(TERMINAL_VIEW_TYPE)) {
    const view = leaf.view;
    if (!(view instanceof TerminalView)) continue;

    const terminal = view.getTerminalInstance();
    if (!terminal || !plugin.isRemoteTerminal(terminal)) continue;

    const nodeId = plugin.getRemoteNodeId(terminal);
    if (!nodeId || !connections.isConnected(nodeId)) continue;

    const device = plugin.getPairedDeviceStore().get(nodeId);
    const label = device ? `${device.name} — ${terminal.getTitle()}` : terminal.getTitle();
    terminals.push({ leaf, view, nodeId, label });
  }

  return terminals;
}

/**
 * Shows a picker at `event`'s position. With no connected terminals, skips
 * straight to the existing "connect a device first" notice instead of
 * opening an empty menu.
 */
export function showTerminalPickerMenu(
  event: MouseEvent,
  terminals: ConnectedTerminal[],
  onPick: (terminal: ConnectedTerminal) => void
): void {
  if (terminals.length === 0) {
    new Notice(t('remote.notConnected'));
    return;
  }

  const menu = new Menu();
  for (const terminal of terminals) {
    menu.addItem((item) => {
      item.setTitle(terminal.label).setIcon('terminal').onClick(() => onPick(terminal));
    });
  }
  menu.showAtMouseEvent(event);
}
