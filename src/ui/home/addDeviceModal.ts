import type { App } from 'obsidian';
import { Modal } from 'obsidian';

import { t } from '../../i18n';
import type { ConnectionCodeProblem } from '../../services/remote/connectionCode';
import type { PairDeviceResult } from '../../services/remote/devicePairing';

export interface AddDeviceModalOptions {
  addDevice: (code: string, name: string) => Promise<PairDeviceResult>;
  onAdded: () => void;
}

export class AddDeviceModal extends Modal {
  private readonly options: AddDeviceModalOptions;
  private deviceName = '';
  private connectionCode = '';
  private codeInputEl: HTMLTextAreaElement | null = null;
  private errorEl: HTMLElement | null = null;
  private submitButtonEl: HTMLButtonElement | null = null;

  constructor(app: App, options: AddDeviceModalOptions) {
    super(app);
    this.options = options;
  }

  onOpen(): void {
    const { contentEl, modalEl } = this;
    modalEl.addClass('termesh-add-device-modal');
    contentEl.empty();
    contentEl.createEl('h2', { text: t('home.addDevice') });

    const nameLabel = contentEl.createEl('label', { cls: 'termesh-form-field' });
    nameLabel.createSpan({ text: t('home.deviceName') });
    const nameInput = nameLabel.createEl('input', {
      type: 'text',
      placeholder: t('home.deviceNamePlaceholder'),
    });
    nameInput.addEventListener('input', () => { this.deviceName = nameInput.value; });

    const codeLabel = contentEl.createEl('label', { cls: 'termesh-form-field' });
    codeLabel.createSpan({ text: t('home.connectionCode') });
    this.codeInputEl = codeLabel.createEl('textarea', {
      placeholder: t('home.connectionCodePlaceholder'),
      attr: { rows: '5' },
    });
    this.codeInputEl.addEventListener('input', () => {
      this.connectionCode = this.codeInputEl?.value ?? '';
      this.setError(null);
    });

    this.errorEl = contentEl.createDiv({ cls: 'termesh-form-error' });
    this.errorEl.setAttribute('role', 'alert');

    const buttons = contentEl.createDiv({ cls: 'modal-button-container' });
    const cancelButton = buttons.createEl('button', { text: t('common.cancel') });
    cancelButton.addEventListener('click', () => this.close());
    this.submitButtonEl = buttons.createEl('button', { cls: 'mod-cta', text: t('home.addDevice') });
    this.submitButtonEl.addEventListener('click', () => { void this.submit(); });

    window.setTimeout(() => this.codeInputEl?.focus(), 0);
  }

  onClose(): void {
    this.contentEl.empty();
  }

  private async submit(): Promise<void> {
    if (!this.submitButtonEl) return;
    this.submitButtonEl.disabled = true;
    this.submitButtonEl.setText(t('home.addingDevice'));
    this.setError(null);
    try {
      const result = await this.options.addDevice(this.connectionCode, this.deviceName);
      if (!result.ok) {
        this.setError(this.getPairingProblemMessage(result.problem));
        this.codeInputEl?.focus();
        return;
      }
      this.options.onAdded();
      this.close();
    } catch (error) {
      this.setError(t('home.operationFailed', {
        message: error instanceof Error ? error.message : String(error),
      }));
    } finally {
      if (this.submitButtonEl) {
        this.submitButtonEl.disabled = false;
        this.submitButtonEl.setText(t('home.addDevice'));
      }
    }
  }

  private setError(message: string | null): void {
    if (!this.errorEl) return;
    this.errorEl.setText(message ?? '');
    this.errorEl.toggleClass('is-visible', message !== null);
  }

  private getPairingProblemMessage(problem: ConnectionCodeProblem | 'unparseable'): string {
    switch (problem) {
      case 'empty': return t('home.pairEmpty');
      case 'wrong-prefix': return t('home.pairWrongPrefix');
      case 'bad-characters': return t('home.pairBadCharacters');
      case 'too-short': return t('home.pairTooShort');
      case 'unparseable': return t('home.pairUnparseable');
    }
  }
}
