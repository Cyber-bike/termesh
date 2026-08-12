/**
 * Directory tree panel (candidate doc "目录树与双向文件传输", phase 1 / local).
 *
 * Scope decisions carried over from the dev doc, worth restating here since
 * they show up directly in this file's shape:
 *  - The tree root is the terminal's current working directory, not some
 *    fixed "workspace root" (there isn't one for an arbitrary terminal
 *    session) or the filesystem root. An "up" affordance re-roots to the
 *    parent directory. Root children render already loaded/expanded;
 *    everything below that is collapsed and lazy-loaded on click.
 *  - Docking is a self-drawn resizable side panel (own CSS class +
 *    drag-resize handle), not Obsidian's native pane-split, and not a true
 *    "grab this button and drop it on an edge" gesture — dock side is a
 *    two-state toggle button instead. See dev doc §4 decision 3 for why.
 *  - The "copy a tree entry into the vault" direction is a context-menu
 *    action, not drag-out. A renderer-only Obsidian plugin has no reliable
 *    way to originate a real OS file drag that Obsidian's own file
 *    explorer (which expects its *own* internal drag payload, not an
 *    arbitrary one) would accept as "create these vault files" — see the
 *    dev doc's front-matter note. Right-click "复制到 Vault" is the
 *    reliable substitute for that direction.
 */

import type { Menu as ObsidianMenu } from 'obsidian';
import { Menu, setIcon } from 'obsidian';

import type { Disposable } from '../../services/remote/transport.ts';
import type { DirectoryEntry, DirectoryTreeSource } from '../../services/terminal/directoryTreeSource.ts';
import { t } from '../../i18n';

export type DockSide = 'left' | 'right';

export interface DirectoryTreePanelCallbacks {
  /** Double-click on a directory node: caller is expected to `cd` the terminal there. */
  onActivateDirectory(path: string): void;
  /** A vault drag landed on a directory node; caller resolves and copies the payload. */
  onDropToPath(dataTransfer: DataTransfer, targetPath: string): void;
  /** "复制到 Vault" was chosen from a node's context menu. */
  onCopyToVault(path: string, isDirectory: boolean, baseName: string): void;
  /** The panel was closed via its own header button. */
  onRequestClose(): void;
}

interface PathApi {
  join(...segments: string[]): string;
  dirname(target: string): string;
  basename(target: string): string;
}

const MIN_WIDTH_PX = 180;
const MAX_WIDTH_PX = 640;
const DEFAULT_WIDTH_PX = 260;

export class DirectoryTreePanel {
  readonly element: HTMLElement;

  private readonly headerEl: HTMLElement;
  private readonly pathLabelEl: HTMLElement;
  private readonly treeRootEl: HTMLElement;
  private readonly resizerEl: HTMLElement;

  private rootPath: string | null = null;
  private dockSide: DockSide = 'right';
  private width = DEFAULT_WIDTH_PX;

  /** One watch subscription per directory currently rendered as expanded. */
  private readonly watches = new Map<string, Disposable>();
  private readonly expandedPaths = new Set<string>();
  private destroyed = false;

  constructor(
    private readonly source: DirectoryTreeSource,
    private readonly pathApi: PathApi,
    private readonly callbacks: DirectoryTreePanelCallbacks,
  ) {
    this.element = createDiv('directory-tree-panel');

    this.headerEl = this.element.createDiv('directory-tree-panel__header');
    this.buildHeaderControls();

    this.pathLabelEl = this.element.createDiv('directory-tree-panel__path');

    this.treeRootEl = this.element.createDiv('directory-tree-panel__tree');

    this.resizerEl = this.element.createDiv('directory-tree-panel__resizer');
    this.bindResizer();

    this.applyDockSide();
    this.applyWidth();
  }

  private buildHeaderControls(): void {
    const upBtn = this.headerEl.createEl('button', {
      cls: 'directory-tree-panel__icon-btn clickable-icon',
      attr: { 'aria-label': t('directoryTree.goUp') },
    });
    setIcon(upBtn, 'arrow-up');
    upBtn.addEventListener('click', () => {
      if (!this.rootPath) return;
      const parent = this.pathApi.dirname(this.rootPath);
      if (parent && parent !== this.rootPath) void this.setRootPath(parent);
    });

    const refreshBtn = this.headerEl.createEl('button', {
      cls: 'directory-tree-panel__icon-btn clickable-icon',
      attr: { 'aria-label': t('directoryTree.refresh') },
    });
    setIcon(refreshBtn, 'refresh-cw');
    refreshBtn.addEventListener('click', () => {
      if (this.rootPath) void this.setRootPath(this.rootPath, { keepExpanded: true });
    });

    const dockBtn = this.headerEl.createEl('button', {
      cls: 'directory-tree-panel__icon-btn clickable-icon',
      attr: { 'aria-label': t('directoryTree.toggleDockSide') },
    });
    setIcon(dockBtn, 'flip-horizontal-2');
    dockBtn.addEventListener('click', () => {
      this.setDockSide(this.dockSide === 'left' ? 'right' : 'left');
    });

    const closeBtn = this.headerEl.createEl('button', {
      cls: 'directory-tree-panel__icon-btn clickable-icon',
      attr: { 'aria-label': t('directoryTree.close') },
    });
    setIcon(closeBtn, 'x');
    closeBtn.addEventListener('click', () => this.callbacks.onRequestClose());
  }

  private bindResizer(): void {
    let startX = 0;
    let startWidth = this.width;
    let dragging = false;

    const onPointerMove = (event: PointerEvent): void => {
      if (!dragging) return;
      // Dragging the handle further from the terminal (left when docked
      // right, right when docked left) grows the panel.
      const delta = this.dockSide === 'right' ? startX - event.clientX : event.clientX - startX;
      this.setWidth(startWidth + delta);
    };

    const onPointerUp = (): void => {
      dragging = false;
      const doc = this.resizerEl.ownerDocument;
      doc.removeEventListener('pointermove', onPointerMove);
      doc.removeEventListener('pointerup', onPointerUp);
    };

    this.resizerEl.addEventListener('pointerdown', (event) => {
      dragging = true;
      startX = event.clientX;
      startWidth = this.width;
      const doc = this.resizerEl.ownerDocument;
      doc.addEventListener('pointermove', onPointerMove);
      doc.addEventListener('pointerup', onPointerUp);
      event.preventDefault();
    });
  }

  private setWidth(px: number): void {
    this.width = Math.min(MAX_WIDTH_PX, Math.max(MIN_WIDTH_PX, px));
    this.applyWidth();
  }

  private applyWidth(): void {
    this.element.style.setProperty('--directory-tree-panel-width', `${this.width}px`);
  }

  getDockSide(): DockSide {
    return this.dockSide;
  }

  setDockSide(side: DockSide): void {
    this.dockSide = side;
    this.applyDockSide();
  }

  private applyDockSide(): void {
    this.element.toggleClass('directory-tree-panel--left', this.dockSide === 'left');
    this.element.toggleClass('directory-tree-panel--right', this.dockSide === 'right');
  }

  async setRootPath(rootPath: string, options: { keepExpanded?: boolean } = {}): Promise<void> {
    this.rootPath = rootPath;
    this.pathLabelEl.setText(rootPath);
    this.pathLabelEl.setAttribute('title', rootPath);

    if (!options.keepExpanded) {
      this.expandedPaths.clear();
    }
    this.disposeAllWatches();
    this.treeRootEl.empty();

    await this.renderChildrenInto(rootPath, this.treeRootEl, 0);
  }

  private async renderChildrenInto(dirPath: string, container: HTMLElement, depth: number): Promise<void> {
    if (this.destroyed) return;
    let entries: DirectoryEntry[];
    try {
      entries = await this.source.list(dirPath);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      container.createDiv({ cls: 'directory-tree-panel__error', text: message });
      return;
    }
    if (this.destroyed) return;

    container.empty();
    this.watchDirectory(dirPath, container, depth);

    if (entries.length === 0) {
      container.createDiv({ cls: 'directory-tree-panel__empty', text: t('directoryTree.empty') });
      return;
    }

    for (const entry of entries) {
      this.renderNode(entry, dirPath, container, depth);
    }
  }

  private watchDirectory(dirPath: string, container: HTMLElement, depth: number): void {
    this.watches.get(dirPath)?.dispose();
    const disposable = this.source.watch(dirPath, () => {
      if (this.destroyed) return;
      void this.renderChildrenInto(dirPath, container, depth);
    });
    this.watches.set(dirPath, disposable);
  }

  private renderNode(entry: DirectoryEntry, parentPath: string, container: HTMLElement, depth: number): void {
    const fullPath = this.pathApi.join(parentPath, entry.name);

    const row = container.createDiv({ cls: 'directory-tree-panel__row' });
    row.style.setProperty('--directory-tree-depth', String(depth));
    row.dataset.path = fullPath;

    const caret = row.createSpan({ cls: 'directory-tree-panel__caret' });
    if (entry.isDirectory) setIcon(caret, 'chevron-right');

    const icon = row.createSpan({ cls: 'directory-tree-panel__node-icon' });
    setIcon(icon, entry.isDirectory ? 'folder' : 'file');

    row.createSpan({ cls: 'directory-tree-panel__node-name', text: entry.name });

    let childrenEl: HTMLElement | null = null;
    let expanded = false;

    const toggle = async (): Promise<void> => {
      if (!entry.isDirectory) return;
      expanded = !expanded;
      row.toggleClass('is-expanded', expanded);
      if (expanded) {
        this.expandedPaths.add(fullPath);
        if (!childrenEl) {
          childrenEl = container.createDiv({ cls: 'directory-tree-panel__children' });
          row.insertAdjacentElement('afterend', childrenEl);
        }
        childrenEl.toggleClass('is-hidden', false);
        await this.renderChildrenInto(fullPath, childrenEl, depth + 1);
      } else {
        this.expandedPaths.delete(fullPath);
        childrenEl?.toggleClass('is-hidden', true);
        this.disposeWatchesUnder(fullPath);
      }
    };

    if (entry.isDirectory) {
      caret.addEventListener('click', (event) => {
        event.stopPropagation();
        void toggle();
      });
      row.addEventListener('dblclick', () => this.callbacks.onActivateDirectory(fullPath));
    }

    row.addEventListener('contextmenu', (event) => {
      event.preventDefault();
      this.showNodeContextMenu(event, fullPath, entry.isDirectory, entry.name);
    });

    if (entry.isDirectory) {
      row.addEventListener('dragover', (event) => {
        event.preventDefault();
        row.addClass('is-drop-target');
      });
      row.addEventListener('dragleave', () => row.removeClass('is-drop-target'));
      row.addEventListener('drop', (event) => {
        event.preventDefault();
        event.stopPropagation();
        row.removeClass('is-drop-target');
        if (event.dataTransfer) this.callbacks.onDropToPath(event.dataTransfer, fullPath);
      });
    }

    if (this.expandedPaths.has(fullPath) && entry.isDirectory) {
      void toggle();
    }
  }

  private showNodeContextMenu(event: MouseEvent, path: string, isDirectory: boolean, name: string): void {
    const menu: ObsidianMenu = new Menu();
    menu.addItem((item) => {
      item
        .setTitle(t('directoryTree.copyToVault'))
        .setIcon('download')
        .onClick(() => this.callbacks.onCopyToVault(path, isDirectory, name));
    });
    menu.showAtMouseEvent(event);
  }

  private disposeWatchesUnder(path: string): void {
    for (const [watchedPath, disposable] of Array.from(this.watches.entries())) {
      if (watchedPath === path || watchedPath.startsWith(`${path}${watchedPath.includes('\\') ? '\\' : '/'}`)) {
        disposable.dispose();
        this.watches.delete(watchedPath);
      }
    }
  }

  private disposeAllWatches(): void {
    for (const disposable of this.watches.values()) disposable.dispose();
    this.watches.clear();
  }

  destroy(): void {
    this.destroyed = true;
    this.disposeAllWatches();
    this.element.remove();
  }
}
