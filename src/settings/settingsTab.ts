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

/**
 * Terminal settings tab class
 */
export class TerminalSettingTab extends PluginSettingTab {
  plugin: TerminalPlugin;
  private terminalRenderer: TerminalSettingsRenderer;
  private expandedSections: Set<string> = new Set();

  constructor(app: App, plugin: TerminalPlugin) {
    super(app, plugin);
    this.plugin = plugin;
    this.terminalRenderer = new TerminalSettingsRenderer();
  }

  display(): void {
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
  }

  private renderRemoteSettings(containerEl: HTMLElement): void {
    const settings = this.plugin.settings.remoteConnection;
    const service = this.plugin.getRemoteService();
    const snapshot = service.getSnapshot();
    if (!snapshot.authenticated) return;

    const section = containerEl.createDiv({ cls: 'terminal-settings-card' });
    new Setting(section).setName(t('remote.title')).setHeading();

    new Setting(section)
      .setName(t('remote.loggedInAs', { login: settings.authSession?.login ?? '' }))
      .addButton((button) => button.setButtonText(t('remote.logout')).onClick(() => {
        service.logout();
        this.display();
      }));

    new Setting(section).setName(t('remote.devices')).addButton((button) => button
      .setButtonText(t('remote.refreshDevices')).onClick(async () => {
        try { await service.refreshDevices(); this.display(); }
        catch (error) { new Notice(error instanceof Error ? error.message : String(error), 5000); }
      }));
    const devices = snapshot.devices;
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
      href: 'https://github.com/jiang-zhong-xi/ReqFirst'
    });
    feedbackContainer.createSpan({ cls: 'settings-feedback-separator', text: ' · ' });
    feedbackContainer.createEl('a', {
      text: t('settings.header.communityLink'),
      href: 'https://t.me/+t6oRqhaw8c1jNzE1'
    });
  }
}

