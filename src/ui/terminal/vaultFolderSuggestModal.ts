/**
 * Destination-folder picker for the directory tree's right-click "复制到
 * Vault" (`main.ts`'s `copyDirectoryTreeEntryToVaultWithPicker`, called
 * without an explicit target folder). Dragging a row directly onto a
 * folder in Obsidian's real file explorer skips this and uses the drop
 * location instead - see `directoryTreePanel.ts`'s doc comment.
 */

import type { App, TFolder } from 'obsidian';
import { FuzzySuggestModal } from 'obsidian';
import { t } from '../../i18n';

/** Vault root is represented as `''` throughout this codebase (see `resolveActiveVaultFolder`). */
const VAULT_ROOT_LABEL = '/';

export class VaultFolderSuggestModal extends FuzzySuggestModal<TFolder | null> {
  private chosen = false;

  constructor(
    app: App,
    private readonly folders: TFolder[],
    private readonly onChoose: (folderPath: string | null) => void,
  ) {
    super(app);
    this.setPlaceholder(t('directoryTree.copyToVaultChooseFolderPlaceholder'));
  }

  getItems(): Array<TFolder | null> {
    return [null, ...this.folders];
  }

  getItemText(item: TFolder | null): string {
    return item ? item.path : VAULT_ROOT_LABEL;
  }

  onChooseItem(item: TFolder | null): void {
    this.chosen = true;
    this.onChoose(item ? item.path : '');
  }

  onClose(): void {
    super.onClose();
    if (!this.chosen) this.onChoose(null);
  }
}

/**
 * Opens the picker and resolves the chosen vault-relative folder path
 * (`''` for the vault root), or `null` if the user dismissed it without
 * choosing - callers should treat `null` as "cancel the copy".
 */
export function pickVaultDestinationFolder(app: App, defaultFolderPath: string): Promise<string | null> {
  const folders = app.vault.getAllFolders(false).sort((a, b) => a.path.localeCompare(b.path));
  return new Promise((resolve) => {
    const modal = new VaultFolderSuggestModal(app, folders, resolve);
    modal.open();
    const defaultQuery = defaultFolderPath || '';
    if (defaultQuery) {
      modal.inputEl.value = defaultQuery;
      modal.inputEl.dispatchEvent(new Event('input'));
    }
  });
}
