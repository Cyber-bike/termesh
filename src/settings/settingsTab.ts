/**
 * Terminal plugin settings tab
 * Provides the terminal configuration interface
 */

import type { App } from 'obsidian';
import { Notice, PluginSettingTab, Setting, setIcon } from 'obsidian';
import type TerminalPlugin from '../main';
import { TerminalSettingsRenderer } from './renderers/terminalSettingsRenderer';
import type { RendererContext } from './types';
import { t } from '../i18n';
import { createTermyLogoSvg } from '../ui/icons';
import { pairDevice } from '../services/remote/devicePairing';
import type { ConnectionCodeProblem } from '../services/remote/connectionCode';
import type { Disposable } from '../services/remote/transport';

/**
 * Terminal settings tab class
 */
export class TerminalSettingTab extends PluginSettingTab {
  plugin: TerminalPlugin;
  private terminalRenderer: TerminalSettingsRenderer;
  private expandedSections: Set<string> = new Set();
  private pairingCode: { id: string; code: string } | null = null;
  private deviceConnectionSubscription: Disposable | null = null;
  private deviceRefreshTimer: number | null = null;
  private v2DeviceName = '';
  private v2ConnectionCode = '';

  constructor(app: App, plugin: TerminalPlugin) {
    super(app, plugin);
    this.plugin = plugin;
    this.terminalRenderer = new TerminalSettingsRenderer();
  }

  display(): void {
    this.deviceConnectionSubscription?.dispose();
    this.deviceConnectionSubscription = null;
    const { containerEl } = this;
    containerEl.empty();

    // Add the main container class
    containerEl.addClass('terminal-settings-container');

    // Render the header section
    this.renderHeader(containerEl);

    // Content container
    const contentEl = containerEl.createDiv({ cls: 'terminal-settings-content' });

    // Create the renderer context
    const context: RendererContext = {
      app: this.app,
      plugin: this.plugin,
      containerEl: contentEl,
      expandedSections: this.expandedSections
    };

    // Render terminal settings
    this.terminalRenderer.render(context);
    this.renderRemoteSettings(contentEl);
    this.renderDirectDeviceSettings(contentEl);
  }

  hide(): void {
    this.deviceConnectionSubscription?.dispose();
    this.deviceConnectionSubscription = null;
    if (this.deviceRefreshTimer !== null) {
      window.clearTimeout(this.deviceRefreshTimer);
      this.deviceRefreshTimer = null;
    }
  }

  private renderRemoteSettings(containerEl: HTMLElement): void {
    const section = containerEl.createDiv({ cls: 'terminal-settings-card' });
    new Setting(section).setName(t('remote.title')).setHeading();
    const settings = this.plugin.settings.remoteConnection;
    const service = this.plugin.getRemoteService();
    let login = '';
    let password = '';

    new Setting(section).setName(t('remote.relayUrl')).setDesc(t('remote.relayUrlDesc')).addText((text) => {
      text.setValue(settings.relayUrl).onChange((value) => {
        settings.relayUrl = value;
        void this.plugin.saveSettings();
      });
    });
    new Setting(section).setName(t('remote.loginName')).addText((text) => text.onChange((value) => { login = value; }));
    new Setting(section).setName(t('remote.password')).addText((text) => {
      text.inputEl.type = 'password';
      text.onChange((value) => { password = value; });
    });
    new Setting(section)
      .addButton((button) => button.setButtonText(t('remote.login')).onClick(async () => {
        try { await service.login(login, password); this.display(); }
        catch (error) { new Notice(error instanceof Error ? error.message : String(error), 5000); }
      }))
      .addButton((button) => button.setButtonText(t('remote.logout')).onClick(() => {
        service.logout();
        this.display();
      }));

    const pairing = new Setting(section).setName(t('remote.pairingCode'));
    pairing.descEl.setText(this.pairingCode?.code ?? '');
    pairing.addButton((button) => button.setButtonText(t('remote.createPairingCode')).onClick(async () => {
      try {
        const created = await service.createPairingCode();
        this.pairingCode = { id: created.pairingCodeId, code: created.pairingCode };
        this.display();
      } catch (error) { new Notice(error instanceof Error ? error.message : String(error), 5000); }
    }));
    pairing.addButton((button) => button.setButtonText(t('remote.revokePairingCode')).setDisabled(!this.pairingCode).onClick(async () => {
      if (!this.pairingCode) return;
      try { await service.revokePairingCode(this.pairingCode.id); this.pairingCode = null; this.display(); }
      catch (error) { new Notice(error instanceof Error ? error.message : String(error), 5000); }
    }));

    new Setting(section).setName(t('remote.devices')).addButton((button) => button
      .setButtonText(t('remote.refreshDevices')).onClick(async () => {
        try { await service.refreshDevices(); this.display(); }
        catch (error) { new Notice(error instanceof Error ? error.message : String(error), 5000); }
      }));
    const devices = service.getSnapshot().devices;
    if (devices.length === 0) section.createDiv({ text: t('remote.noDevices') });
    for (const device of devices) {
      new Setting(section).setName(device.name)
        .setDesc(`${device.platform} · ${device.online ? t('remote.states.Connected') : t('remote.offline')}`)
        .addButton((button) => button.setButtonText(t('common.delete')).onClick(async () => {
          try { await service.deleteDevice(device.id); this.display(); }
          catch (error) { new Notice(error instanceof Error ? error.message : String(error), 5000); }
        }));
    }
  }

  private renderDirectDeviceSettings(containerEl: HTMLElement): void {
    const section = containerEl.createDiv({ cls: 'terminal-settings-card' });
    new Setting(section).setName(t('remote.v2.title')).setDesc(t('remote.v2.description')).setHeading();

    new Setting(section).setName(t('remote.v2.deviceName')).addText((text) => {
      text
        .setPlaceholder(t('remote.v2.deviceNamePlaceholder'))
        .setValue(this.v2DeviceName)
        .onChange((value) => { this.v2DeviceName = value; });
    });

    new Setting(section).setName(t('remote.v2.connectionCode')).addTextArea((text) => {
      text
        .setPlaceholder(t('remote.v2.connectionCodePlaceholder'))
        .setValue(this.v2ConnectionCode)
        .onChange((value) => { this.v2ConnectionCode = value; });
    }).addButton((button) => button.setButtonText(t('remote.v2.addDevice')).setCta().onClick(async () => {
      button.setDisabled(true).setButtonText(t('remote.v2.addingDevice'));
      try {
        const module = await this.plugin.loadIroh();
        const result = pairDevice(
          this.plugin.getPairedDeviceStore(),
          (code) => ({
            nodeId: module.EndpointTicket.fromString(code).endpointAddr().id().toString(),
          }),
          this.v2ConnectionCode,
          this.v2DeviceName,
        );
        if (!result.ok) {
          new Notice(this.getPairingProblemMessage(result.problem), 5000);
          return;
        }

        this.v2DeviceName = '';
        this.v2ConnectionCode = '';
        await this.plugin.saveSettings();
        new Notice(t('remote.v2.deviceAdded'));
        this.display();
      } catch (error) {
        this.showRemoteOperationError(error);
      } finally {
        button.setDisabled(false).setButtonText(t('remote.v2.addDevice'));
      }
    }));

    const store = this.plugin.getPairedDeviceStore();
    const connections = this.plugin.getDeviceConnectionManager();
    this.deviceConnectionSubscription = connections.onDidChange(() => {
      if (this.deviceRefreshTimer !== null) return;
      this.deviceRefreshTimer = window.setTimeout(() => {
        this.deviceRefreshTimer = null;
        if (this.containerEl.isConnected) this.display();
      }, 0);
    });

    const devices = store.list();
    if (devices.length === 0) {
      section.createDiv({ text: t('remote.noDevices') });
      return;
    }

    for (const device of devices) {
      const status = connections.status(device.nodeId);
      const statusText = this.getDeviceStatusText(status.state);
      const lastConnected = device.lastConnectedAt
        ? t('remote.v2.lastConnected', { time: this.formatLastConnectedAt(device.lastConnectedAt) })
        : t('remote.v2.neverConnected');
      const row = new Setting(section)
        .setName(device.name)
        .setDesc(`${statusText} · ${lastConnected}`);

      row.addButton((button) => button
        .setButtonText(status.state === 'connected' ? t('remote.disconnect') : t('remote.connect'))
        .setDisabled(status.state === 'connecting')
        .onClick(async () => {
          try {
            if (status.state === 'connected') {
              connections.disconnect(device.nodeId);
            } else {
              await connections.connect(device.nodeId);
            }
          } catch (error) {
            this.showRemoteOperationError(error);
          }
        }));
      row.addButton((button) => button
        .setButtonText(t('remote.v2.newTerminal'))
        .setDisabled(status.state === 'connecting')
        .onClick(async () => {
          try {
            await this.plugin.openRemoteTerminal(device.nodeId);
          } catch (error) {
            this.showRemoteOperationError(error);
          }
        }));
      row.addButton((button) => button
        .setButtonText(t('remote.v2.removeDevice'))
        .setWarning()
        .onClick(async () => {
          connections.disconnect(device.nodeId);
          store.remove(device.nodeId);
          await this.plugin.saveSettings();
          new Notice(t('remote.v2.deviceRemoved'));
          this.display();
        }));
    }
  }

  private getPairingProblemMessage(problem: ConnectionCodeProblem | 'unparseable'): string {
    switch (problem) {
      case 'empty': return t('remote.v2.pairEmpty');
      case 'wrong-prefix': return t('remote.v2.pairWrongPrefix');
      case 'bad-characters': return t('remote.v2.pairBadCharacters');
      case 'too-short': return t('remote.v2.pairTooShort');
      case 'unparseable': return t('remote.v2.pairUnparseable');
    }
  }

  private getDeviceStatusText(state: 'disconnected' | 'connecting' | 'connected' | 'error'): string {
    switch (state) {
      case 'disconnected': return t('remote.v2.statusDisconnected');
      case 'connecting': return t('remote.v2.statusConnecting');
      case 'connected': return t('remote.v2.statusConnected');
      case 'error': return t('remote.v2.statusError');
    }
  }

  private formatLastConnectedAt(value: string): string {
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
  }

  private showRemoteOperationError(error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    new Notice(t('remote.v2.operationFailed', { message }), 7000);
  }

  /**
   * Render the header section
   */
  private renderHeader(containerEl: HTMLElement): void {
    const headerEl = containerEl.createDiv({ cls: 'terminal-settings-header settings-header' });

    // Title row (includes the icon, title, changelog button, and reload button)
    const titleRow = headerEl.createDiv({ cls: 'settings-title-row' });

    // Left side: logo + title + changelog button
    const titleGroup = titleRow.createDiv({ cls: 'settings-title-group' });
    
    // Add the Termy logo
    const iconContainer = titleGroup.createDiv({ cls: 'settings-title-icon' });
    iconContainer.appendChild(createTermyLogoSvg(32));

    titleGroup.createDiv({ cls: 'settings-title', text: t('settings.header.title') });

    const changelogBtn = titleGroup.createEl('button', {
      cls: 'settings-header-button settings-title-changelog-button',
    });
    changelogBtn.setAttribute('type', 'button');
    setIcon(changelogBtn, 'scroll-text');
    changelogBtn.createSpan({ text: t('settings.header.changelog') });
    changelogBtn.addEventListener('click', () => {
      this.plugin.showChangelog();
    });

    // Right side: feedback link + reload button
    const actionsGroup = titleRow.createDiv({ cls: 'settings-actions-group' });
    
    const feedbackContainer = actionsGroup.createDiv({ cls: 'settings-feedback' });
    feedbackContainer.appendText(t('settings.header.feedbackText'));
    feedbackContainer.createEl('a', {
      text: t('settings.header.feedbackLink'),
      href: 'https://github.com/ZyphrZero/Termy'
    });
    feedbackContainer.createSpan({ cls: 'settings-feedback-separator', text: ' · ' });
    feedbackContainer.createEl('a', {
      text: t('settings.header.communityLink'),
      href: 'https://t.me/+t6oRqhaw8c1jNzE1'
    });
  }
}

