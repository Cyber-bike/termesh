import type { WorkspaceLeaf } from 'obsidian';
import { ItemView, Notice, setIcon, setTooltip } from 'obsidian';

import type TerminalPlugin from '../../main';
import { t } from '../../i18n';
import { buildDeviceHomeCards, getRefreshNodeIds, type DeviceHomeCard } from '../../services/remote/deviceHomeModel';
import { pairDevice, type PairDeviceResult } from '../../services/remote/devicePairing';
import type { Disposable } from '../../services/remote/transport';
import { AddDeviceModal } from './addDeviceModal';
import { RemoveDeviceModal } from './removeDeviceModal';

export const DEVICE_HOME_VIEW_TYPE = 'termesh-device-home';

export class DeviceHomeView extends ItemView {
  private readonly plugin: TerminalPlugin;
  private connectionSubscription: Disposable | null = null;
  private renderTimer: number | null = null;
  private refreshing = false;

  constructor(leaf: WorkspaceLeaf, plugin: TerminalPlugin) {
    super(leaf);
    this.plugin = plugin;
  }

  getViewType(): string {
    return DEVICE_HOME_VIEW_TYPE;
  }

  getDisplayText(): string {
    return t('plugin.name');
  }

  getIcon(): string {
    return 'monitor-dot';
  }

  onOpen(): Promise<void> {
    const connections = this.plugin.getDeviceConnectionManager();
    this.connectionSubscription = connections.onDidChange(() => this.scheduleRender());
    this.render();
    return Promise.resolve();
  }

  onClose(): Promise<void> {
    this.connectionSubscription?.dispose();
    this.connectionSubscription = null;
    if (this.renderTimer !== null) {
      window.clearTimeout(this.renderTimer);
      this.renderTimer = null;
    }
    return Promise.resolve();
  }

  private render(): void {
    const container = this.contentEl;
    container.empty();
    container.addClass('termesh-device-home');

    const header = container.createDiv({ cls: 'termesh-home-header' });
    const heading = header.createDiv({ cls: 'termesh-home-heading' });
    heading.createEl('h1', { text: t('home.title') });
    heading.createEl('p', { text: t('home.description') });

    const refreshButton = header.createEl('button', {
      cls: 'clickable-icon termesh-home-refresh',
      attr: { 'aria-label': this.refreshing ? t('home.refreshing') : t('home.refresh') },
    });
    setIcon(refreshButton, 'refresh-cw');
    setTooltip(refreshButton, this.refreshing ? t('home.refreshing') : t('home.refresh'));
    refreshButton.disabled = this.refreshing;
    refreshButton.toggleClass('is-refreshing', this.refreshing);
    refreshButton.addEventListener('click', () => { void this.refreshDevices(); });

    const connections = this.plugin.getDeviceConnectionManager();
    const cards = buildDeviceHomeCards(this.plugin.getPairedDeviceStore().list(), connections);
    const grid = container.createDiv({ cls: 'termesh-device-grid' });
    for (const card of cards) this.renderCard(grid, card);
  }

  private renderCard(grid: HTMLElement, card: DeviceHomeCard): void {
    if (card.kind === 'add') {
      const cardEl = this.createInteractiveCard(grid, 'termesh-device-card is-add', t('home.addDevice'), () => {
        this.openAddDeviceModal();
      });
      const icon = cardEl.createDiv({ cls: 'termesh-device-icon' });
      setIcon(icon, 'plus');
      cardEl.createEl('h2', { text: t('home.addDevice') });
      cardEl.createEl('p', { text: t('home.addDeviceDescription') });
      return;
    }

    if (card.kind === 'local') {
      const cardEl = this.createInteractiveCard(grid, 'termesh-device-card is-local', t('home.openTerminal'), () => {
        void this.plugin.openLocalDeviceTerminal();
      });
      const icon = cardEl.createDiv({ cls: 'termesh-device-icon' });
      setIcon(icon, 'monitor');
      cardEl.createEl('h2', { text: t('home.localDevice') });
      cardEl.createEl('p', { text: t('home.localDeviceDescription') });
      this.renderStatus(cardEl, 'connected', t('home.available'));
      return;
    }

    const { device, status } = card;
    const cardEl = this.createInteractiveCard(
      grid,
      `termesh-device-card is-remote status-${status.state}`,
      `${device.name}: ${this.getStatusText(status.state)}`,
      () => {
        if (status.state !== 'connecting') void this.openRemoteTerminal(device.nodeId);
      },
      status.state === 'connecting',
    );
    const top = cardEl.createDiv({ cls: 'termesh-device-card-top' });
    const icon = top.createDiv({ cls: 'termesh-device-icon' });
    setIcon(icon, 'server');
    const actions = top.createDiv({ cls: 'termesh-device-actions' });
    if (status.state === 'connected') {
      this.createIconButton(actions, 'unplug', t('home.disconnect'), () => {
        this.plugin.getDeviceConnectionManager().disconnect(device.nodeId);
      });
    }
    this.createIconButton(actions, 'trash-2', t('home.removeDevice'), () => {
      new RemoveDeviceModal(this.app, device.name, async () => {
        this.plugin.getDeviceConnectionManager().disconnect(device.nodeId);
        this.plugin.getPairedDeviceStore().remove(device.nodeId);
        await this.plugin.saveSettings();
        new Notice(t('home.deviceRemoved'));
        this.render();
      }).open();
    });

    cardEl.createEl('h2', { text: device.name });
    const lastConnected = device.lastConnectedAt
      ? t('home.lastConnected', { time: this.formatLastConnectedAt(device.lastConnectedAt) })
      : t('home.neverConnected');
    cardEl.createEl('p', { text: lastConnected });
    this.renderStatus(cardEl, status.state, this.getStatusText(status.state));
    if (status.state === 'error') {
      cardEl.createDiv({ cls: 'termesh-device-error', text: status.message });
    }
  }

  private createInteractiveCard(
    parent: HTMLElement,
    className: string,
    label: string,
    activate: () => void,
    disabled = false,
  ): HTMLElement {
    const card = parent.createDiv({ cls: className });
    card.setAttribute('role', 'button');
    card.setAttribute('aria-label', label);
    card.setAttribute('tabindex', disabled ? '-1' : '0');
    card.toggleClass('is-disabled', disabled);
    card.addEventListener('click', (event) => {
      if (!disabled && !(event.target as HTMLElement).closest('button')) activate();
    });
    card.addEventListener('keydown', (event) => {
      if (!disabled && event.target === card && (event.key === 'Enter' || event.key === ' ')) {
        event.preventDefault();
        activate();
      }
    });
    return card;
  }

  private createIconButton(parent: HTMLElement, iconName: string, label: string, action: () => void): void {
    const button = parent.createEl('button', { cls: 'clickable-icon', attr: { 'aria-label': label } });
    setIcon(button, iconName);
    setTooltip(button, label);
    button.addEventListener('click', (event) => {
      event.stopPropagation();
      action();
    });
  }

  private renderStatus(parent: HTMLElement, state: string, label: string): void {
    const status = parent.createDiv({ cls: `termesh-device-status status-${state}` });
    status.createSpan({ cls: 'termesh-status-dot' });
    status.createSpan({ text: label });
  }

  private openAddDeviceModal(): void {
    new AddDeviceModal(this.app, {
      addDevice: (code, name) => this.addDevice(code, name),
      onAdded: () => {
        new Notice(t('home.deviceAdded'));
        this.render();
      },
    }).open();
  }

  private async addDevice(code: string, name: string): Promise<PairDeviceResult> {
    const module = await this.plugin.loadIroh();
    const result = pairDevice(
      this.plugin.getPairedDeviceStore(),
      (normalizedCode) => ({
        nodeId: module.EndpointTicket.fromString(normalizedCode).endpointAddr().id().toString(),
      }),
      code,
      name,
    );
    if (result.ok) await this.plugin.saveSettings();
    return result;
  }

  private async openRemoteTerminal(nodeId: string): Promise<void> {
    try {
      await this.plugin.openRemoteTerminal(nodeId, this.plugin.getActiveNoteName());
    } catch (error) {
      new Notice(t('home.operationFailed', {
        message: error instanceof Error ? error.message : String(error),
      }), 7000);
    }
  }

  private async refreshDevices(): Promise<void> {
    if (this.refreshing) return;
    this.refreshing = true;
    this.render();
    const connections = this.plugin.getDeviceConnectionManager();
    const cards = buildDeviceHomeCards(this.plugin.getPairedDeviceStore().list(), connections);
    await Promise.allSettled(getRefreshNodeIds(cards).map((nodeId) => connections.connect(nodeId)));
    this.refreshing = false;
    this.render();
  }

  private scheduleRender(): void {
    if (this.renderTimer !== null) return;
    this.renderTimer = window.setTimeout(() => {
      this.renderTimer = null;
      if (this.contentEl.isConnected) this.render();
    }, 0);
  }

  private getStatusText(state: 'disconnected' | 'connecting' | 'connected' | 'error'): string {
    switch (state) {
      case 'disconnected': return t('home.statusDisconnected');
      case 'connecting': return t('home.statusConnecting');
      case 'connected': return t('home.statusConnected');
      case 'error': return t('home.statusError');
    }
  }

  private formatLastConnectedAt(value: string): string {
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
  }
}
