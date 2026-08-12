import type { App } from 'obsidian';
import { Modal } from 'obsidian';

import { t } from '../../i18n';

export class RemoveDeviceModal extends Modal {
  private readonly deviceName: string;
  private readonly onConfirm: () => Promise<void>;

  constructor(app: App, deviceName: string, onConfirm: () => Promise<void>) {
    super(app);
    this.deviceName = deviceName;
    this.onConfirm = onConfirm;
  }

  onOpen(): void {
    this.contentEl.empty();
    this.contentEl.createEl('h2', { text: t('home.removeDeviceTitle') });
    this.contentEl.createEl('p', { text: t('home.removeDeviceDescription', { name: this.deviceName }) });
    const actions = this.contentEl.createDiv({ cls: 'modal-button-container' });
    const cancelButton = actions.createEl('button', { text: t('common.cancel') });
    cancelButton.addEventListener('click', () => this.close());
    const removeButton = actions.createEl('button', { cls: 'mod-warning', text: t('home.removeDevice') });
    removeButton.addEventListener('click', () => {
      removeButton.disabled = true;
      void this.onConfirm().then(() => this.close()).finally(() => { removeButton.disabled = false; });
    });
  }

  onClose(): void {
    this.contentEl.empty();
  }
}