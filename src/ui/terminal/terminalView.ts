import type { WorkspaceLeaf, Menu } from 'obsidian';
import { FileSystemAdapter, ItemView, Notice, TFile, TFolder, setIcon } from 'obsidian';
import { shell, webUtils } from 'electron';
import { WebLinksAddon } from '@xterm/addon-web-links';

/**
 * Node built-ins are resolved on demand inside the `TerminalView`
 * constructor via Electron's `window.require` to keep filesystem
 * access out of the bundled module top-level scope. This avoids
 * tripping the Obsidian community plugin reviewer's static "Direct
 * Filesystem Access" warning while preserving runtime semantics
 * (Electron caches `require` results).
 */
type FsModule = typeof import('fs');
type PathModule = typeof import('path');

import type { TerminalService } from '../../services/terminal/terminalService';
import type { TerminalConnectionStatus, TerminalInstance } from '../../services/terminal/terminalInstance';
import {
  collectFallbackDroppedTextPayload,
  collectPreferredDroppedTextPayload,
  resolveDroppedTextInput,
} from '../../services/terminal/dropTextPayload';
import { formatClaudeCodePathReferences } from '../../services/terminal/claudeCodePathReferences';
import {
  collectTerminalReferenceCandidatePaths,
  fileUriToPlatformPath,
  findUniqueTerminalEntryByBasename,
  getVaultRelativePathFromAbsolute,
  isBasenameOnlyTerminalToken,
  isAbsoluteTerminalPath,
  isWindowsStylePath,
  joinTerminalPaths,
  normalizeDroppedEntryReference,
  normalizeDroppedMarkdownLinkpath,
  normalizeTerminalRawToken,
  normalizeTerminalReferencePath,
  normalizeTerminalToken,
  normalizeVaultPath,
  obsidianUriToVaultPath,
  toPlatformPath,
} from '../../services/terminal/terminalPathUtils';
import { TERMINAL_FILE_URI_REGEX } from '../../services/terminal/terminalFileLinks';
import type { TerminalSettings } from '../../settings/settings';
import { debugLog, errorLog } from '../../utils/logger';
import { clamp, normalizeBackgroundPosition, normalizeBackgroundSize, toCssUrl } from '../../utils/styleUtils';
import { t } from '../../i18n';
import { RenameTerminalModal } from './renameTerminalModal';
import { capabilities, transition, type RemoteState } from '../../services/remote/remoteState';
import { createVaultLinkSource, readVaultFile } from '../../services/remote/vaultLinkSource';
import { checkQuotas, collect } from '../../services/remote/noteCollector';
import { DirectoryTreePanel } from './directoryTreePanel';
import { LocalDirectoryTreeSource } from '../../services/terminal/directoryTreeSource';
import type { DirectoryTreeSource } from '../../services/terminal/directoryTreeSource';
import {
  collectVaultEntryForTransfer,
  copyVaultEntryToDirectory,
  copyVaultNoteWithLinksToDirectory,
  type DirectoryTreeDragPayload,
  type FsAccess,
} from '../../services/terminal/directoryTreeDrop';
import type { DeviceConnectionManager } from '../../services/remote/deviceConnections';
import { getHomeDir, isWindows } from '../../utils/platform';
type XtermTerminal = import('@xterm/xterm').Terminal;

export const TERMINAL_VIEW_TYPE = 'terminal-view';

export type TerminalAttachOptions = {
  focus?: boolean;
};

/**
 * Terminal view class
 */
export class TerminalView extends ItemView {
  protected terminalService: TerminalService | null;
  private terminalInstance: TerminalInstance | null = null;
  private terminalContainer: HTMLElement | null = null;
  private dropHintEl: HTMLElement | null = null;
  private dropCursorHintEl: HTMLElement | null = null;
  private dragEnterDepth = 0;
  private removeDropHandlers: (() => void) | null = null;
  private searchContainer: HTMLElement | null = null;
  private searchInput: HTMLInputElement | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private fileUriLinkAddon: WebLinksAddon | null = null;
  private titleChangeCleanup: (() => void) | null = null;
  private searchStateCleanup: (() => void) | null = null;
  private connectionStatusCleanup: (() => void) | null = null;
  private initPromise: Promise<TerminalInstance> | null = null;
  private initResolve: ((terminal: TerminalInstance) => void) | null = null;
  private initReject: ((error: Error) => void) | null = null;
  private remoteState: RemoteState = 'LocalMode';
  private remoteToolbar: HTMLElement | null = null;
  private connectionStatus: TerminalConnectionStatus | 'reconnecting' = 'disconnected';

  private terminalBody: HTMLElement | null = null;
  private directoryTreePanel: DirectoryTreePanel | null = null;
  private directoryTreeVisible = false;

  private readonly fs: FsModule;
  private readonly path: PathModule;

  constructor(leaf: WorkspaceLeaf, terminalService: TerminalService | null) {
    super(leaf);
    this.terminalService = terminalService;
    this.fs = window.require('fs') as FsModule;
    this.path = window.require('path') as PathModule;
    this.initPromise = new Promise<TerminalInstance>((resolve, reject) => {
      this.initResolve = resolve;
      this.initReject = reject;
    });
  }

  getViewType(): string { return TERMINAL_VIEW_TYPE; }

  getDisplayText(): string {
    return this.terminalInstance?.getTitle() || t('terminal.defaultTitle');
  }

  getIcon(): string { return 'terminal'; }

  onPaneMenu(menu: Menu): void {
    // Obsidian may pass a wrapper object, so resolve the real view instance
    const view = (this as TerminalView & { realView?: TerminalView }).realView ?? this;
    
    menu.addItem((item) => {
      item.setTitle(t('terminal.renameTerminal'))
        .setIcon('pencil')
        .onClick(() => {
          if (!view.terminalInstance) {
            new Notice(t('terminal.notInitialized'));
            return;
          }
          
          const currentTitle = view.terminalInstance.getTitle();
          
          new RenameTerminalModal(
            view.app,
            currentTitle,
            (newTitle: string) => {
              if (view.terminalInstance && newTitle.trim()) {
                const trimmedTitle = newTitle.trim();
                view.terminalInstance.setTitle(trimmedTitle);
                this.updateLeafHeader(view.leaf);
                view.updateDropHintText();
              }
            }
          ).open();
        });
    });

    const plugin = this.getTerminalPlugin();
    if (plugin) {
      menu.addItem((item) => {
        item.setTitle(plugin.getAlwaysOnTopTerminalLabel(view))
          .setIcon('pin')
          .onClick(() => {
            void plugin.toggleAlwaysOnTopTerminal(view);
          });
      });
    }
  }

  onOpen(): Promise<void> {
    // Use contentEl instead of containerEl.children[1]
    const container = this.contentEl;
    container.empty();
    container.addClass('terminal-view-container');

    // Create the search bar container
    this.searchContainer = container.createDiv('terminal-search-container');
    this.createSearchUI();

    this.remoteToolbar = container.createDiv('terminal-remote-toolbar');
    this.renderRemoteToolbar();

    this.terminalBody = container.createDiv('terminal-body');
    this.terminalContainer = this.terminalBody.createDiv('terminal-container');
    this.ensureDropHint();
    this.hideDropHint();
    if (!this.removeDropHandlers) {
      this.removeDropHandlers = this.setupDropHandlers();
    }

    window.setTimeout(() => {
      if (!this.terminalInstance && this.terminalContainer) {
        void this.initializeTerminal();
      }
    }, 0);
    return Promise.resolve();
  }

  /**
   * Create the search UI
   */
  private createSearchUI(): void {
    if (!this.searchContainer) return;

    // Search input
    this.searchInput = activeDocument.createElement('input');
    this.searchInput.type = 'text';
    this.searchInput.placeholder = t('terminal.search.placeholder');
    this.searchInput.className = 'terminal-search-input';

    // Search input handler
    this.searchInput.addEventListener('input', () => {
      this.performSearch();
    });

    this.searchInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        if (e.shiftKey) {
          this.terminalInstance?.searchPrevious();
        } else {
          this.terminalInstance?.searchNext();
        }
      } else if (e.key === 'Escape') {
        this.hideSearch();
      }
    });

    this.searchContainer.appendChild(this.searchInput);

    // Previous button
    const prevBtn = this.createSearchButton('chevron-up', t('terminal.search.previous'), () => {
      this.terminalInstance?.searchPrevious();
    });
    this.searchContainer.appendChild(prevBtn);

    // Next button
    const nextBtn = this.createSearchButton('chevron-down', t('terminal.search.next'), () => {
      this.terminalInstance?.searchNext();
    });
    this.searchContainer.appendChild(nextBtn);

    // Close button
    const closeBtn = this.createSearchButton('x', t('terminal.search.close'), () => {
      this.hideSearch();
    });
    this.searchContainer.appendChild(closeBtn);
  }

  /**
   * Create a search button
   */
  private createSearchButton(icon: string, title: string, onClick: () => void): HTMLElement {
    const btn = activeDocument.createElement('button');
    btn.className = 'terminal-search-btn clickable-icon';
    btn.title = title;
    setIcon(btn, icon);
    btn.addEventListener('click', onClick);
    return btn;
  }

  /**
   * Perform a search
   */
  private performSearch(): void {
    const query = this.searchInput?.value || '';
    this.terminalInstance?.search(query);
  }

  /**
   * Show the search bar
   */
  showSearch(): void {
    if (this.searchContainer) {
      this.searchContainer.addClass('is-visible');
      this.searchInput?.focus();
      this.searchInput?.select();
    }
  }

  /**
   * Hide the search bar
   */
  hideSearch(): void {
    if (this.searchContainer) {
      this.searchContainer.removeClass('is-visible');
    }
    this.terminalInstance?.clearSearch();
    this.terminalInstance?.focus();
  }

  async onClose(): Promise<void> {
    this.getTerminalPlugin()?.handleTerminalViewClosed(this);

    this.resizeObserver?.disconnect();
    this.resizeObserver = null;
    this.fileUriLinkAddon?.dispose();
    this.fileUriLinkAddon = null;
    this.titleChangeCleanup?.();
    this.titleChangeCleanup = null;
    this.searchStateCleanup?.();
    this.searchStateCleanup = null;
    this.connectionStatusCleanup?.();
    this.connectionStatusCleanup = null;
    this.removeDropHandlers?.();
    this.removeDropHandlers = null;
    this.dragEnterDepth = 0;
    this.dropHintEl = null;
    this.dropCursorHintEl?.remove();
    this.dropCursorHintEl = null;
    this.directoryTreePanel?.destroy();
    this.directoryTreePanel = null;
    this.directoryTreeVisible = false;
    this.terminalBody = null;

    if (this.terminalInstance) {
      try {
        await this.terminalService?.destroyTerminal(this.terminalInstance.id);
      } catch (error) {
        errorLog('[TerminalView] Destroy failed:', error);
      }
      this.terminalInstance = null;
    }

    this.containerEl.empty();
    this.disposeAppearanceStyle();
  }

  releaseTerminalInstance(): TerminalInstance | null {
    const terminal = this.terminalInstance;
    if (!terminal) return null;

    this.detachTerminalBindings();
    this.fileUriLinkAddon?.dispose();
    this.fileUriLinkAddon = null;
    terminal.detach();
    this.terminalInstance = null;
    this.initPromise = Promise.resolve(terminal);
    this.initResolve = null;
    this.initReject = null;
    return terminal;
  }

  adoptTerminalInstance(terminal: TerminalInstance, options: TerminalAttachOptions = {}): void {
    this.detachTerminalBindings();
    this.terminalInstance = terminal;
    this.remoteState = this.getTerminalPlugin()?.isRemoteTerminal(terminal) ? 'Connected' : 'LocalMode';
    this.connectionStatus = 'connected';
    this.initPromise = Promise.resolve(terminal);
    this.initResolve?.(terminal);
    this.initResolve = null;
    this.initReject = null;
    this.bindTerminalInstance(terminal);
    this.registerTerminalHyperlinkHandler(terminal.getXterm());
    this.updateAppearanceStyles();
    this.attachTerminalToContainer(options);
    this.setupResizeObserver();
    this.updateLeafHeader(this.leaf);
    this.updateDropHintText();
    // A directory tree panel built before this swap (e.g. a reconnect
    // replacing the underlying TerminalInstance, `main.ts`'s
    // `reconnectTerminalView`) still holds the *previous* instance's
    // DirectoryTreeSource and remote-node id baked into its constructor, and
    // its rootPath was computed from the old instance's cwd. Left alone, it
    // silently keeps browsing/showing stale (or, if remote-ness changed,
    // outright wrong-machine) data instead of the newly adopted terminal's.
    // Rebuild it against the new instance rather than leaving it stale.
    if (this.directoryTreePanel) {
      const wasVisible = this.directoryTreeVisible;
      this.closeDirectoryTree();
      if (wasVisible) this.openDirectoryTree();
    }
    this.renderRemoteToolbar();
  }

  setTerminalService(terminalService: TerminalService): void {
    this.terminalService = terminalService;
    this.renderRemoteToolbar();
  }

  handleHostWindowChanged(options: TerminalAttachOptions = {}): void {
    if (!this.terminalInstance || !this.terminalContainer) return;

    this.removeDropHandlers?.();
    this.removeDropHandlers = this.setupDropHandlers();
    this.updateAppearanceStyles();
    this.attachTerminalToContainer(options);
    this.setupResizeObserver();
  }

  private async initializeTerminal(): Promise<void> {
    try {
      if (!this.terminalService) {
        throw new Error('TerminalService not initialized');
      }

      this.terminalInstance = await this.terminalService.createTerminal();
      this.initResolve?.(this.terminalInstance);
      this.initResolve = null;
      this.initReject = null;

      this.bindTerminalInstance(this.terminalInstance);
      const xterm = this.terminalInstance.getXterm();
      this.registerTerminalHyperlinkHandler(xterm);

      this.updateAppearanceStyles();
      this.attachTerminalToContainer();
      this.setupResizeObserver();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      errorLog('[TerminalView] Init failed:', errorMessage);
      if (this.initReject) {
        this.initReject(error instanceof Error ? error : new Error(errorMessage));
        this.initResolve = null;
        this.initReject = null;
      }
      new Notice(t('notices.terminal.initFailed', { message: errorMessage }));
      this.leaf.detach();
    }
  }

  /**
   * Create a new terminal
   */
  private async createNewTerminal(): Promise<void> {
    // Trigger the plugin's activateTerminalView method
    // Get the plugin instance through the workspace
    const plugin = this.getTerminalPlugin();
    if (plugin) {
      await plugin.activateTerminalView();
    }
  }

  private bindTerminalInstance(terminal: TerminalInstance): void {
    this.detachTerminalBindings();
    this.titleChangeCleanup = terminal.onTitleChange(() => {
      this.updateLeafHeader(this.leaf);
      this.updateDropHintText();
    });

    this.searchStateCleanup = terminal.onSearchStateChange((visible) => {
      if (visible) {
        this.showSearch();
      } else {
        this.hideSearch();
      }
    });

    this.connectionStatusCleanup = terminal.onConnectionStatusChange((status) => {
      this.connectionStatus = status;
      this.renderRemoteToolbar();
    });

    terminal.setOnNewTerminal(() => {
      void this.createNewTerminal();
    });

    terminal.setOnSplitTerminal((direction) => {
      void this.splitTerminal(direction);
    });

    terminal.setOnToggleAlwaysOnTop(
      () => {
        const plugin = this.getTerminalPlugin();
        if (plugin) {
          void plugin.toggleAlwaysOnTopTerminal(this);
        }
      },
      () => this.getTerminalPlugin()?.getAlwaysOnTopTerminalLabel(this) ?? t('terminal.contextMenu.pinToTop')
    );

    terminal.setDefaultShellMenuCallbacks(
      () => this.terminalService?.getDefaultShellOptions() ?? [],
      (shellType) => {
        void this.terminalService?.setDefaultShell(shellType).catch((error) => {
          const message = error instanceof Error ? error.message : String(error);
          errorLog('[TerminalView] Failed to switch default shell:', error);
          new Notice(message);
        });
      }
    );
  }

  private detachTerminalBindings(): void {
    this.titleChangeCleanup?.();
    this.titleChangeCleanup = null;
    this.searchStateCleanup?.();
    this.searchStateCleanup = null;
    this.connectionStatusCleanup?.();
    this.connectionStatusCleanup = null;
  }

  /**
   * Split the terminal (used by commands)
   */
  async splitTerminal(direction: 'horizontal' | 'vertical'): Promise<void> {
    const { workspace } = this.app;
    const newLeaf = workspace.getLeaf('split', direction);
    
    await newLeaf.setViewState({
      type: TERMINAL_VIEW_TYPE,
      active: true,
    });

    workspace.setActiveLeaf(newLeaf, { focus: true });
  }

  private setupDropHandlers(): () => void {
    const container = this.contentEl;
    const cleanup: Array<() => void> = [];
    const capture = false;
    const dragWindow = container.ownerDocument?.defaultView;

    const addListener = (
      target: EventTarget,
      type: string,
      listener: EventListenerOrEventListenerObject
    ): void => {
      target.addEventListener(type, listener, capture);
      cleanup.push(() => target.removeEventListener(type, listener, capture));
    };

    const claimDragEvent = (event: DragEvent): void => {
      event.preventDefault();
      if (event.dataTransfer) {
        event.dataTransfer.dropEffect = 'copy';
      }
    };

    const onDragEnter = (event: DragEvent): void => {
      claimDragEvent(event);
      this.dragEnterDepth += 1;
      this.showDropHintForState(event);
    };

    const onDragOver = (event: DragEvent): void => {
      claimDragEvent(event);
      this.showDropHintForState(event);
    };

    const onDragLeave = (event: DragEvent): void => {
      claimDragEvent(event);
      this.dragEnterDepth = Math.max(0, this.dragEnterDepth - 1);
      const relatedTarget = event.relatedTarget as Node | null;
      const leftContainer = !relatedTarget || !container.contains(relatedTarget);
      if (this.dragEnterDepth === 0 || leftContainer) {
        this.dragEnterDepth = 0;
        this.hideAllDropHints();
      }
    };

    const onDrop = (event: DragEvent): void => {
      claimDragEvent(event);
      this.resetDropHintState();
      void this.handleDrop(event.dataTransfer);
    };

    const onWindowDragEnd = (): void => {
      this.resetDropHintState();
    };

    addListener(container, 'dragenter', onDragEnter);
    addListener(container, 'dragover', onDragOver);
    addListener(container, 'dragleave', onDragLeave);
    addListener(container, 'drop', onDrop);

    if (dragWindow) {
      addListener(dragWindow, 'dragend', onWindowDragEnd);
    }

    return () => {
      for (const dispose of cleanup.splice(0)) {
        dispose();
      }
      this.dropCursorHintEl?.remove();
      this.dropCursorHintEl = null;
    };
  }

  private ensureDropHint(): void {
    if (!this.terminalContainer) return;
    if (this.dropHintEl && this.dropHintEl.isConnected) return;

    const doc = this.terminalContainer.ownerDocument;
    const hint = doc.createElement('div');
    hint.className = 'terminal-drop-hint';
    const textEl = doc.createElement('div');
    textEl.className = 'terminal-drop-hint__text';
    hint.appendChild(textEl);
    this.dropHintEl = hint;
    this.updateDropHintText();
    this.terminalContainer.appendChild(hint);
  }

  private getDropHintText(): string {
    // A connected remote terminal only accepts a single Markdown note (the
    // same constraint `resolveDroppedMarkdownFile` enforces on drop), so
    // warn about it up front instead of showing the LocalMode-oriented
    // paste hint, which never applies here.
    if (this.remoteState === 'Connected') {
      return t('remote.dropSingleMarkdown');
    }
    return t('terminal.dropHintPasteFilePath');
  }

  private updateDropHintText(): void {
    if (!this.dropHintEl) return;
    const textEl = this.dropHintEl.querySelector('.terminal-drop-hint__text');
    if (textEl) {
      textEl.textContent = this.getDropHintText();
      return;
    }
    this.dropHintEl.textContent = this.getDropHintText();
  }

  private showDropHint(): void {
    this.ensureDropHint();
    if (!this.dropHintEl?.classList.contains('is-visible')) {
      this.updateDropHintText();
    }
    this.dropHintEl?.classList.add('is-visible');
  }

  private hideDropHint(): void {
    this.dropHintEl?.classList.remove('is-visible');
  }

  private ensureDropCursorHint(): void {
    if (this.dropCursorHintEl && this.dropCursorHintEl.isConnected) return;
    const doc = this.contentEl.ownerDocument;
    const hint = doc.createElement('div');
    hint.className = 'terminal-drop-cursor-hint';
    this.dropCursorHintEl = hint;
    doc.body.appendChild(hint);
  }

  private positionDropCursorHint(event: DragEvent): void {
    if (!this.dropCursorHintEl) return;
    const offset = 16;
    this.dropCursorHintEl.style.left = `${event.clientX + offset}px`;
    this.dropCursorHintEl.style.top = `${event.clientY + offset}px`;
  }

  private showDropCursorHint(event: DragEvent): void {
    this.ensureDropCursorHint();
    if (this.dropCursorHintEl) {
      this.dropCursorHintEl.textContent = this.getDropHintText();
    }
    this.positionDropCursorHint(event);
    this.dropCursorHintEl?.classList.add('is-visible');
  }

  private hideDropCursorHint(): void {
    this.dropCursorHintEl?.classList.remove('is-visible');
  }

  /**
   * Connected mode sends the dropped note to the remote device rather than
   * typing it into the terminal, so covering the whole terminal with the
   * paste-oriented full-screen hint (`showDropHint`) wrongly frames the
   * drop as terminal input. A small tooltip that tracks the cursor keeps
   * the "this note goes to the device" framing instead of "this becomes
   * terminal text" without blocking the terminal view underneath.
   */
  private showDropHintForState(event: DragEvent): void {
    if (this.remoteState === 'Connected') {
      this.hideDropHint();
      this.showDropCursorHint(event);
      return;
    }
    this.hideDropCursorHint();
    this.showDropHint();
  }

  private hideAllDropHints(): void {
    this.hideDropHint();
    this.hideDropCursorHint();
  }

  private resetDropHintState(): void {
    this.dragEnterDepth = 0;
    this.hideAllDropHints();
  }

  private async handleDrop(dataTransfer: DataTransfer | null): Promise<void> {
    if (this.remoteState === 'Connected') {
      await this.handleRemoteDrop(dataTransfer);
      return;
    }
    if (this.remoteState !== 'LocalMode') {
      new Notice(t('remote.notConnected'));
      return;
    }
    const input = await this.buildDroppedInput(dataTransfer);
    if (!input) {
      debugLog('[Terminal DnD] No usable file path or text in drop payload');
      errorLog('[Terminal DnD] No usable path details:', this.describeDropPayload(dataTransfer));
      new Notice('Termesh: 未获取到可用文本或路径，请确认拖拽来源是否支持文本或文件。');
      return;
    }

    debugLog('[Terminal DnD] Inject input:', input.text);
    await this.writeInputToTerminal(input.text, input.usePaste);
  }

  private async handleRemoteDrop(dataTransfer: DataTransfer | null): Promise<void> {
    const file = this.resolveDroppedMarkdownFile(dataTransfer);
    if (!file) {
      new Notice(t('remote.dropRejected'));
      return;
    }

    // The terminal's `remoteState` is driven by the device connection this
    // terminal actually rides (see `isRemoteTerminal`), not by the legacy
    // relay-based `RemoteService` - so the transfer must go out over the
    // same `DeviceConnectionManager` connection, matching how the directory
    // tree already sends dropped vault entries (`sendVaultEntriesToRemote`).
    const nodeId = this.getRemoteNodeId();
    const connections = nodeId ? this.getTerminalPlugin()?.getDeviceConnectionManager() : null;
    if (!nodeId || !connections) {
      new Notice(t('remote.notConnected'));
      return;
    }

    // No directory was explicitly chosen (unlike a directory-tree drop), so
    // land the note in the terminal's current directory instead of leaving
    // it up to the agent's generic receive folder - and tell the user where
    // that is, the same way `dropCopyDone` already reports a directory-tree
    // drop's destination.
    const targetPath = this.getRemoteDropTargetPath();

    this.setRemoteState(transition(this.remoteState, { type: 'dropNote' }));
    try {
      const collected = collect(createVaultLinkSource(this.app, file));
      if (!collected.ok) throw new Error(collected.error ?? 'Unable to collect dropped note');
      const quota = checkQuotas(collected.files);
      if (!quota.ok) throw new Error(quota.error ?? 'Transfer quota exceeded');

      const outcome = await connections
        .createTransferSender(nodeId, crypto.randomUUID(), collected.files, (path) => readVaultFile(this.app, path), null, targetPath)
        .run();
      if (!outcome.success) throw new Error(outcome.message || 'Transfer failed');
      new Notice(t('remote.transferCompleteAt', { path: targetPath }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      new Notice(t('remote.transferFailed', { message }), 5000);
    } finally {
      this.setRemoteState(transition(this.remoteState, { type: 'transferFinished' }));
    }
  }

  /**
   * Best-known landing directory for a note dropped straight onto the
   * terminal (no directory-tree target chosen). Mirrors `openDirectoryTree`'s
   * cwd-detection: `getCwd()` falls back to the *local* initial cwd when the
   * remote shell hasn't reported one yet, which would be a nonsense path on
   * the remote OS, so that case instead uses `~` - the same sentinel the
   * agent already expands to the real home directory.
   *
   * Public because `sendNoteRecursively` (v3.1's right-click/toolbar send
   * entries) reuses this exact priority for terminals it did not itself
   * receive a drop on.
   */
  getRemoteDropTargetPath(): string {
    const terminal = this.terminalInstance;
    if (!terminal) return '~';
    const cwd = terminal.getCwd();
    if (!cwd || cwd === terminal.getInitialCwd()) return '~';
    return cwd;
  }

  private resolveDroppedMarkdownFile(dataTransfer: DataTransfer | null): TFile | null {
    if (!dataTransfer || dataTransfer.files.length > 1) return null;
    const candidates = [
      dataTransfer.getData('text/plain'),
      dataTransfer.getData('text/uri-list'),
      ...Array.from(dataTransfer.files).map((file) => file.name),
    ].flatMap((candidate) => candidate.split(/\r?\n/));
    for (const candidate of candidates) {
      const path = normalizeDroppedMarkdownLinkpath(candidate);
      if (!path) continue;
      const direct = this.app.vault.getAbstractFileByPath(normalizeVaultPath(path));
      if (direct instanceof TFile && direct.extension.toLowerCase() === 'md') return direct;
      const linked = this.app.metadataCache.getFirstLinkpathDest(path, '');
      if (linked instanceof TFile && linked.extension.toLowerCase() === 'md') return linked;
    }
    return null;
  }

  /**
   * Like `resolveDroppedMarkdownFile` but not restricted to a single
   * Markdown note: the directory tree accepts any note, attachment, or
   * folder dragged from the vault (candidate doc §4.1 point 5). Files and
   * folders resolve the same way `normalizeDroppedMarkdownLinkpath` already
   * resolves note links; the name is inherited from that helper, not a
   * markdown restriction baked into its logic.
   */
  private resolveDroppedVaultEntries(dataTransfer: DataTransfer | null): Array<TFile | TFolder> {
    if (!dataTransfer) return [];
    const candidates = [
      dataTransfer.getData('text/plain'),
      dataTransfer.getData('text/uri-list'),
      ...Array.from(dataTransfer.files).map((file) => file.name),
    ].flatMap((candidate) => candidate.split(/\r?\n/));

    const results: Array<TFile | TFolder> = [];
    const seenPaths = new Set<string>();
    for (const candidate of candidates) {
      const path = normalizeDroppedMarkdownLinkpath(candidate);
      if (!path) continue;
      const direct = this.app.vault.getAbstractFileByPath(normalizeVaultPath(path));
      const resolved = direct instanceof TFile || direct instanceof TFolder
        ? direct
        : this.app.metadataCache.getFirstLinkpathDest(path, '');
      if ((resolved instanceof TFile || resolved instanceof TFolder) && !seenPaths.has(resolved.path)) {
        seenPaths.add(resolved.path);
        results.push(resolved);
      }
    }
    return results;
  }

  private buildDirectoryTreeFsAccess(): FsAccess {
    return {
      promises: this.fs.promises,
      join: (...segments: string[]) => this.path.join(...segments),
    };
  }

  /**
   * A remote directory tree lists a device that may not share the local
   * machine's OS, so joining/splitting its paths with the local `path`
   * module (win32 on a Windows control machine, say, joining a Linux
   * agent's paths) mangles separators and breaks every `fsList` past the
   * first level. Node's `path` module always exposes `.win32`/`.posix`
   * regardless of the host OS, so each call picks the one matching what the
   * path itself looks like rather than assuming it matches the local OS.
   */
  private buildDirectoryTreePathApi(isRemote: boolean): { join(...segments: string[]): string; dirname(target: string): string; basename(target: string): string } {
    if (!isRemote) {
      return {
        join: (...segments: string[]) => this.path.join(...segments),
        dirname: (target: string) => this.path.dirname(target),
        basename: (target: string) => this.path.basename(target),
      };
    }
    const moduleFor = (sample: string) => (isWindowsStylePath(sample) ? this.path.win32 : this.path.posix);
    return {
      join: (...segments: string[]) => moduleFor(segments[0] ?? '').join(...segments),
      dirname: (target: string) => moduleFor(target).dirname(target),
      basename: (target: string) => moduleFor(target).basename(target),
    };
  }

  toggleDirectoryTree(): void {
    if (this.directoryTreeVisible) {
      this.closeDirectoryTree();
    } else {
      this.openDirectoryTree();
    }
  }

  /** The connected device's nodeId if the current terminal is remote, else null. */
  private getRemoteNodeId(): string | null {
    if (!this.terminalInstance) return null;
    return this.getTerminalPlugin()?.getRemoteNodeId(this.terminalInstance) ?? null;
  }

  private buildDirectoryTreeSource(): DirectoryTreeSource {
    const nodeId = this.getRemoteNodeId();
    const connections = nodeId ? this.getTerminalPlugin()?.getDeviceConnectionManager() : null;
    if (nodeId && connections) {
      return connections.createDirectoryTreeSource(nodeId);
    }
    return new LocalDirectoryTreeSource(this.fs);
  }

  private openDirectoryTree(): void {
    if (!this.terminalBody) return;

    if (!this.directoryTreePanel) {
      const plugin = this.getTerminalPlugin();
      this.directoryTreePanel = new DirectoryTreePanel(
        this.buildDirectoryTreeSource(),
        this.buildDirectoryTreePathApi(this.getRemoteNodeId() !== null),
        {
          onActivateDirectory: (path) => this.activateDirectoryFromTree(path),
          onDropToPath: (dataTransfer, targetPath) => void this.handleDirectoryTreeDrop(dataTransfer, targetPath),
          onCopyToVault: (path, isDirectory, baseName) => void this.handleCopyToVault(path, isDirectory, baseName),
          onRequestClose: () => this.closeDirectoryTree(),
          onDockSideChange: (side) => {
            const current = this.getTerminalPlugin();
            if (!current) return;
            current.settings.directoryTreeDockSide = side;
            void current.saveSettings();
          },
        },
        plugin?.settings.directoryTreeDockSide ?? 'right',
        this.getRemoteNodeId(),
      );
    }

    if (!this.directoryTreePanel.element.isConnected) {
      this.terminalBody.appendChild(this.directoryTreePanel.element);
    }

    this.directoryTreeVisible = true;
    this.renderRemoteToolbar();

    const terminal = this.terminalInstance;
    const nodeId = this.getRemoteNodeId();
    let rootPath = terminal?.getCwd() ?? getHomeDir();

    // Remote terminals can only browse the agent's filesystem, so avoid
    // falling back to the local vault path when no remote cwd has been
    // detected yet. Use the remote home sentinel instead; the agent expands
    // it to the real home directory.
    if (nodeId && terminal) {
      const initialCwd = terminal.getInitialCwd();
      if (!terminal.getCwd() || rootPath === initialCwd) {
        rootPath = '~';
      }
    }

    void this.directoryTreePanel.setRootPath(rootPath);
  }

  private closeDirectoryTree(): void {
    this.directoryTreePanel?.destroy();
    this.directoryTreePanel = null;
    this.directoryTreeVisible = false;
    this.renderRemoteToolbar();
  }

  private activateDirectoryFromTree(path: string): void {
    const terminal = this.terminalInstance;
    if (!terminal) return;
    const quoted = /\s/.test(path) ? `"${path}"` : path;
    // A remote terminal's OS can differ from the local machine's, so the
    // path's own shape decides which `cd` syntax to send, not `isWindows()`
    // (that only describes the machine running Obsidian).
    const targetIsWindows = this.getRemoteNodeId() !== null ? isWindowsStylePath(path) : isWindows();
    const command = targetIsWindows ? `cd /d ${quoted}` : `cd ${quoted}`;
    terminal.sendText(`${command}\r`);
    terminal.focus();
  }

  private async handleDirectoryTreeDrop(dataTransfer: DataTransfer, targetPath: string): Promise<void> {
    const entries = this.resolveDroppedVaultEntries(dataTransfer);
    if (entries.length === 0) {
      new Notice(t('directoryTree.dropRejectedNotVaultItem'));
      return;
    }

    const nodeId = this.getRemoteNodeId();
    try {
      const allSkippedNotes: Array<{ path: string; reason: string }> = [];
      if (nodeId) {
        await this.sendVaultEntriesToRemote(nodeId, entries, targetPath);
      } else {
        const fsAccess = this.buildDirectoryTreeFsAccess();
        for (const entry of entries) {
          // A dropped Markdown note also pulls in every note/attachment it
          // links to, recursively, preserving each one's vault-relative
          // path under the target directory - matches the "send to
          // terminal" toolbar action's recursive behavior for the remote
          // case (`sendNoteRecursively`), just landing on the local disk
          // instead of over the wire.
          if (entry instanceof TFile && entry.extension.toLowerCase() === 'md') {
            const result = await copyVaultNoteWithLinksToDirectory(this.app, entry, targetPath, fsAccess);
            allSkippedNotes.push(...result.skippedNotes);
          } else {
            await copyVaultEntryToDirectory(this.app, entry, targetPath, fsAccess);
          }
        }
      }
      new Notice(t('directoryTree.dropCopyDone', { path: targetPath }));
      if (allSkippedNotes.length > 0) {
        const details = allSkippedNotes.map((s) => `${s.path}: ${s.reason}`).join('; ');
        new Notice(t('remote.linkedNotesSkipped', { details }), 8000);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      new Notice(t('directoryTree.dropCopyFailed', { message }), 5000);
    }
  }

  /** Sends each dropped vault entry to `targetPath` on the connected device (candidate doc §4.1 point 4). */
  private async sendVaultEntriesToRemote(
    nodeId: string,
    entries: Array<TFile | TFolder>,
    targetPath: string,
  ): Promise<void> {
    const connections = this.getTerminalPlugin()?.getDeviceConnectionManager();
    if (!connections) throw new Error('Remote connection is not available');

    for (const entry of entries) {
      const { files, readFile } = collectVaultEntryForTransfer(entry);
      if (files.length === 0) continue;
      const outcome = await connections
        .createTransferSender(nodeId, crypto.randomUUID(), files, readFile, null, targetPath)
        .run();
      if (!outcome.success) throw new Error(outcome.message || 'Transfer failed');
    }
  }

  /**
   * The right-click "复制到 Vault" entry point. Dragging a row out to a
   * folder in Obsidian's real file explorer is the other entry point to
   * the same underlying copy - see `main.ts`'s explorer-drop listener,
   * which calls the plugin-level `copyDirectoryTreeEntryToVaultWithPicker`
   * directly (with a folder already resolved from where the drop landed)
   * since that isn't tied to any one open terminal view.
   */
  private async handleCopyToVault(absolutePath: string, isDirectory: boolean, baseName: string): Promise<void> {
    const entry: DirectoryTreeDragPayload = {
      path: absolutePath,
      isDirectory,
      baseName,
      nodeId: this.getRemoteNodeId(),
    };
    await this.getTerminalPlugin()?.copyDirectoryTreeEntryToVaultWithPicker(entry);
  }

  private setRemoteState(state: RemoteState): void {
    this.remoteState = state;
    this.terminalInstance?.setInputEnabled(capabilities(state).input);
    this.renderRemoteToolbar();
  }

  private renderRemoteToolbar(): void {
    const toolbar = this.remoteToolbar;
    if (!toolbar) return;
    toolbar.empty();
    const reconnectButton = toolbar.createEl('button', { text: t('terminal.reconnect') });
    reconnectButton.disabled = !this.terminalInstance || this.connectionStatus === 'reconnecting';
    reconnectButton.addEventListener('click', () => void this.reconnectTerminal());

    const treeToggleBtn = toolbar.createEl('button', {
      cls: 'terminal-directory-tree-toggle',
      text: t('directoryTree.toggle'),
      attr: { 'aria-label': t('commands.terminalToggleDirectoryTree') },
    });
    treeToggleBtn.toggleClass('is-active', this.directoryTreeVisible);
    treeToggleBtn.addEventListener('click', () => this.toggleDirectoryTree());

    toolbar.createSpan({
      cls: `terminal-connection-status is-${this.connectionStatus}`,
      text: t(`terminal.connectionStatus.${this.connectionStatus}`),
    });
  }

  private async reconnectTerminal(): Promise<void> {
    const plugin = this.getTerminalPlugin();
    if (!plugin || !this.terminalInstance || this.connectionStatus === 'reconnecting') return;
    this.connectionStatus = 'reconnecting';
    this.renderRemoteToolbar();
    try {
      await plugin.reconnectTerminalView(this);
      this.connectionStatus = 'connected';
    } catch (error) {
      this.connectionStatus = 'error';
      const message = error instanceof Error ? error.message : String(error);
      new Notice(message, 5000);
    }
    this.renderRemoteToolbar();
  }

  private async buildDroppedInput(dataTransfer: DataTransfer | null): Promise<{ text: string; usePaste: boolean } | null> {
    if (!dataTransfer) return null;

    const droppedItems = Array.from(dataTransfer.items);
    const nativePaths = this.extractDroppedNativePaths(dataTransfer);
    if (nativePaths.length > 0) {
      return {
        text: this.formatDroppedPaths(nativePaths),
        usePaste: false,
      };
    }
    const primaryTextPayload = collectPreferredDroppedTextPayload(dataTransfer);
    const fallbackTextPayload = await collectFallbackDroppedTextPayload(dataTransfer, droppedItems);
    return resolveDroppedTextInput(
      primaryTextPayload,
      fallbackTextPayload,
      (payload) => this.extractDroppedPathsFromTextPayload(payload),
      (paths) => this.formatDroppedPaths(paths)
    );
  }

  private extractDroppedNativePaths(dataTransfer: DataTransfer | null): string[] {
    if (!dataTransfer) return [];

    const paths: string[] = [];
    const droppedFiles = Array.from(dataTransfer.files);
    const droppedItems = Array.from(dataTransfer.items);

    for (const item of droppedItems) {
      const itemPath = (item as DataTransferItem & { path?: string }).path;
      if (typeof itemPath === 'string' && itemPath.trim().length > 0) {
        paths.push(itemPath.trim());
      }

      const itemFile = item.getAsFile();
      if (itemFile) {
        const droppedPath = this.getDroppedFilePath(itemFile);
        if (droppedPath) {
          paths.push(droppedPath);
        }
      }

      const entryPath = this.getPathFromDroppedEntry(item);
      if (entryPath) {
        paths.push(entryPath);
      }
    }

    for (const file of droppedFiles) {
      const filePath = this.getDroppedFilePath(file);
      if (filePath) {
        paths.push(filePath);
      }
    }

    return this.uniquePaths(paths);
  }

  private extractDroppedPathsFromTextPayload(textPayload = ''): string[] {
    const paths: string[] = [];

    for (const token of this.extractDropTokens(textPayload)) {
      const resolvedPath = this.resolveDroppedTokenToPath(token);
      if (resolvedPath) paths.push(resolvedPath);
    }

    return this.uniquePaths(paths);
  }

  private describeDropPayload(dataTransfer: DataTransfer | null): Record<string, unknown> {
    if (!dataTransfer) {
      return { hasDataTransfer: false };
    }

    const items = Array.from(dataTransfer.items).map((item) => ({
      kind: item.kind,
      type: item.type,
      hasEntry: !!item.webkitGetAsEntry(),
      entryIsDirectory: !!item.webkitGetAsEntry()?.isDirectory,
      path: (item as DataTransferItem & { path?: string }).path ?? null,
    }));

    const files = Array.from(dataTransfer.files).map((file) => ({
      name: file.name,
      size: file.size,
      type: file.type,
      path: this.getDroppedFilePath(file),
    }));

    return {
      hasDataTransfer: true,
      types: Array.from(dataTransfer.types),
      files,
      items,
    };
  }

  private getDroppedFilePath(file: File & { path?: string }): string | null {
    if (typeof file.path === 'string' && file.path.trim().length > 0) {
      return toPlatformPath(file.path);
    }

    try {
      const resolvedPath = webUtils?.getPathForFile?.(file);
      if (typeof resolvedPath === 'string' && resolvedPath.trim().length > 0) {
        return toPlatformPath(resolvedPath);
      }
    } catch (error) {
      debugLog('[Terminal DnD] webUtils.getPathForFile failed:', error);
    }

    return null;
  }

  private getPathFromDroppedEntry(item: DataTransferItem): string | null {
    const entry = item.webkitGetAsEntry();
    if (!entry) return null;

    const entryPath = entry.fullPath ?? '';
    const normalizedEntry = normalizeDroppedEntryReference(entryPath);
    if (normalizedEntry.absolutePath && this.fs.existsSync(normalizedEntry.absolutePath)) {
      return normalizedEntry.absolutePath;
    }

    const vaultPath = normalizedEntry.vaultPath ?? normalizeVaultPath(entryPath);
    if (vaultPath) {
      const absoluteVaultPath = this.resolveVaultReferenceToAbsolute(vaultPath);
      if (absoluteVaultPath) {
        return absoluteVaultPath;
      }
    }

    if (normalizedEntry.absolutePath) {
      return normalizedEntry.absolutePath;
    }

    return null;
  }

  private extractDropTokens(text: string): string[] {
    if (!text) return [];

    const lineTokens = text
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line.length > 0 && !line.startsWith('#'));

    const uriTokens = Array.from(text.matchAll(/(?:obsidian|file):\/\/[^\s<>"'`]+/g)).map((match) => match[0]);

    return Array.from(new Set([...lineTokens, ...uriTokens]));
  }

  private resolveDroppedTokenToPath(token: string): string | null {
    const rawToken = normalizeTerminalRawToken(token);
    if (!rawToken) return null;

    const obsidianPath = this.obsidianUriToAbsolutePath(rawToken);
    if (obsidianPath) return obsidianPath;

    const fileUriPath = fileUriToPlatformPath(rawToken);
    if (fileUriPath) return fileUriPath;

    const normalized = normalizeTerminalToken(token);
    if (!normalized) return null;

    const wikiMatch = normalized.match(/^\[\[([^\]|#]+)(?:#[^\]|]+)?(?:\|[^\]]+)?\]\]$/);
    if (wikiMatch) {
      return this.resolveVaultReferenceToAbsolute(wikiMatch[1]);
    }

    if (isAbsoluteTerminalPath(normalized)) {
      return toPlatformPath(normalized);
    }

    if (isBasenameOnlyTerminalToken(normalized)) {
      const basenamePath = this.resolveUniqueVaultBasenameToAbsolute(normalized);
      if (basenamePath) {
        return basenamePath;
      }
    }

    return this.resolveVaultReferenceToAbsolute(normalized, true);
  }

  private quoteDroppedPaths(paths: string[]): string {
    return paths.map((path) => `"${path.replace(/"/g, '\\"')}"`).join(' ');
  }

  private formatDroppedPaths(paths: string[]): string {
    if (!this.shouldFormatDroppedPathsAsClaudeCodeReferences()) {
      return this.quoteDroppedPaths(paths);
    }

    return formatClaudeCodePathReferences(paths, {
      cwd: this.terminalInstance?.getCwd(),
      isDirectory: (path) => this.isDroppedDirectoryPath(path),
      pathExists: (path) => this.fs.existsSync(path),
    });
  }

  private shouldFormatDroppedPathsAsClaudeCodeReferences(): boolean {
    const terminal = this.terminalInstance;
    if (!terminal) {
      return false;
    }

    return terminal.isClaudeCodeSession();
  }

  private isDroppedDirectoryPath(path: string): boolean {
    try {
      return this.fs.statSync(path).isDirectory();
    } catch {
      return false;
    }
  }

  private isDropEventInsideContainer(event: DragEvent, container: HTMLElement): boolean {
    const target = event.target;
    if (target instanceof Node && container.contains(target)) {
      return true;
    }

    const rect = container.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) {
      return false;
    }

    return event.clientX >= rect.left
      && event.clientX <= rect.right
      && event.clientY >= rect.top
      && event.clientY <= rect.bottom;
  }

  private uniquePaths(paths: string[]): string[] {
    const result: string[] = [];
    const seen = new Set<string>();

    for (const rawPath of paths) {
      const normalized = rawPath.trim();
      if (!normalized) continue;
      const key = process.platform === 'win32' ? normalized.toLowerCase() : normalized;
      if (seen.has(key)) continue;
      seen.add(key);
      result.push(normalized);
    }

    return result;
  }

  private obsidianUriToAbsolutePath(uri: string): string | null {
    const vaultPath = obsidianUriToVaultPath(uri);
    return vaultPath ? this.resolveVaultPathToAbsolute(vaultPath) : null;
  }

  private resolveVaultPathToAbsolute(pathLike: string): string | null {
    const normalizedPath = normalizeVaultPath(pathLike);
    if (!normalizedPath) return null;

    const activePath = this.app.workspace.getActiveFile()?.path ?? '';
    // Prefer an exact vault entry so folder drops are not shadowed by folder notes.
    const entry = this.app.vault.getAbstractFileByPath(normalizedPath)
      ?? this.app.metadataCache.getFirstLinkpathDest(normalizedPath, activePath);
    if (!entry) return null;

    const adapter = this.app.vault.adapter;
    if (!(adapter instanceof FileSystemAdapter)) {
      return entry.path;
    }

    return joinTerminalPaths(adapter.getBasePath(), entry.path);
  }

  private resolveVaultReferenceToAbsolute(pathLike: string, allowBasenameFallback = false): string | null {
    return this.resolveVaultPathToAbsolute(pathLike)
      ?? (allowBasenameFallback ? this.resolveUniqueVaultBasenameToAbsolute(pathLike) : null);
  }

  private resolveUniqueVaultBasenameToAbsolute(name: string): string | null {
    const allEntries = this.app.vault.getAllLoadedFiles?.() ?? [];
    const matchedEntry = findUniqueTerminalEntryByBasename(name, allEntries.map((entry) => ({
      name: entry.name,
      path: entry.path,
      kind: entry instanceof TFolder ? 'folder' : 'file' as const,
    })));

    if (!matchedEntry) {
      return null;
    }

    const adapter = this.app.vault.adapter;
    if (!(adapter instanceof FileSystemAdapter)) {
      return matchedEntry.path;
    }

    return joinTerminalPaths(adapter.getBasePath(), matchedEntry.path);
  }

  private async writeInputToTerminal(text: string, usePaste = false): Promise<void> {
    const terminal = this.terminalInstance ?? await this.waitForTerminalInstance().catch(() => null);
    if (!terminal) return;
    if (usePaste) {
      terminal.pasteText(text);
    } else {
      terminal.sendText(text);
    }
    terminal.focus();
  }

  private registerTerminalHyperlinkHandler(xterm: XtermTerminal): void {
    xterm.options.linkHandler = {
      allowNonHttpProtocols: true,
      activate: (event: MouseEvent, target: string) => {
        event.preventDefault();
        void this.openTerminalHyperlinkTarget(target);
      },
    };

    this.fileUriLinkAddon?.dispose();
    this.fileUriLinkAddon = new WebLinksAddon((event, uri) => {
      event.preventDefault();
      void this.openTerminalHyperlinkTarget(uri);
    }, {
      urlRegex: TERMINAL_FILE_URI_REGEX,
    });
    xterm.loadAddon(this.fileUriLinkAddon);
  }

  private async openTerminalHyperlinkTarget(target: string): Promise<void> {
    const filePath = fileUriToPlatformPath(target);
    if (filePath) {
      await this.openTerminalFileReference(filePath);
      return;
    }

    if (!this.isAllowedExternalHyperlink(target)) {
      new Notice(t('notices.terminal.fileReferenceUnavailable'));
      return;
    }

    try {
      await shell.openExternal(target);
    } catch (error) {
      errorLog('[TerminalView] Failed to open terminal hyperlink:', target, error);
      new Notice(t('notices.terminal.fileReferenceOpenFailed'));
    }
  }

  private isAllowedExternalHyperlink(target: string): boolean {
    try {
      const url = new URL(normalizeTerminalToken(target));
      return url.protocol === 'http:' || url.protocol === 'https:';
    } catch {
      return false;
    }
  }

  private async openTerminalFileReference(pathLike: string): Promise<void> {
    const resolved = this.resolveTerminalFileReference(pathLike);
    if (!resolved) {
      new Notice(t('notices.terminal.fileReferenceUnavailable'));
      return;
    }

    if (resolved.file) {
      await this.openVaultFileReference(resolved.file);
      return;
    }

    const errorMessage = await shell.openPath(resolved.externalPath);
    if (errorMessage) {
      if (this.fs.existsSync(resolved.externalPath)) {
        const containingDir = this.path.dirname(resolved.externalPath);
        const directoryError = await shell.openPath(containingDir);
        if (!directoryError) {
          return;
        }
      }

      errorLog('[TerminalView] Failed to open external path:', resolved.externalPath, errorMessage);
      new Notice(t('notices.terminal.fileReferenceOpenFailed'));
    }
  }

  private resolveTerminalFileReference(pathLike: string): { file?: TFile; externalPath: string } | null {
    const normalizedReference = normalizeTerminalReferencePath(pathLike);
    if (!normalizedReference) {
      return null;
    }

    if (isAbsoluteTerminalPath(normalizedReference)) {
      const fileFromAbsolutePath = this.absolutePathToVaultFile(normalizedReference);
      if (fileFromAbsolutePath) {
        return {
          file: fileFromAbsolutePath,
          externalPath: normalizedReference,
        };
      }

      if (!this.fs.existsSync(normalizedReference)) {
        return null;
      }

      return { externalPath: normalizedReference };
    }

    const vaultFile = this.resolveVaultReference(normalizedReference);
    if (vaultFile) {
      return {
        file: vaultFile,
        externalPath: vaultFile.path,
      };
    }

    for (const absolutePath of this.getTerminalReferenceAbsoluteCandidates(normalizedReference)) {
      const fileFromCandidate = this.absolutePathToVaultFile(absolutePath);
      if (fileFromCandidate) {
        return {
          file: fileFromCandidate,
          externalPath: absolutePath,
        };
      }

      if (this.fs.existsSync(absolutePath)) {
        return { externalPath: absolutePath };
      }
    }

    return null;
  }

  private resolveVaultReference(pathLike: string): TFile | null {
    const normalizedPath = normalizeVaultPath(pathLike);
    if (!normalizedPath) {
      return null;
    }

    const activePath = this.app.workspace.getActiveFile()?.path ?? '';
    const file = this.app.metadataCache.getFirstLinkpathDest(normalizedPath, activePath)
      ?? this.app.vault.getAbstractFileByPath(normalizedPath);

    return file instanceof TFile ? file : null;
  }

  private absolutePathToVaultFile(absolutePath: string): TFile | null {
    const adapter = this.app.vault.adapter;
    if (!(adapter instanceof FileSystemAdapter)) {
      return null;
    }

    const relativePath = getVaultRelativePathFromAbsolute(absolutePath, adapter.getBasePath());
    if (relativePath === null) {
      return null;
    }

    const file = this.app.vault.getAbstractFileByPath(relativePath);
    return file instanceof TFile ? file : null;
  }

  private getTerminalReferenceAbsoluteCandidates(relativePath: string): string[] {
    const adapter = this.app.vault.adapter;
    const vaultBasePath = adapter instanceof FileSystemAdapter ? adapter.getBasePath() : null;
    const currentCwd = this.terminalInstance?.getCwd() ?? null;
    const initialCwd = this.terminalInstance?.getInitialCwd() ?? null;

    return collectTerminalReferenceCandidatePaths(
      relativePath,
      [currentCwd, initialCwd, vaultBasePath],
    );
  }

  private async openVaultFileReference(file: TFile): Promise<void> {
    const leaf = this.app.workspace.getLeaf(false);
    await leaf.openFile(file);
    this.app.workspace.setActiveLeaf(leaf, { focus: true });
  }

  private updateAppearanceStyles(): void {
    if (!this.terminalContainer || !this.terminalInstance) return;

    const options = this.terminalInstance.getOptions();
    const canUseBackgroundImage = !!options?.backgroundImage
      && !options?.useObsidianTheme
      && this.terminalInstance.getCurrentRenderer() !== 'webgl';

    if (canUseBackgroundImage) {
      this.terminalContainer.addClass('has-background-image');
      this.containerEl.querySelector('.terminal-view-container')?.addClass('has-background-image');
      this.ensureBackgroundLayer();
    } else {
      this.terminalContainer.removeClass('has-background-image');
      this.containerEl.querySelector('.terminal-view-container')?.removeClass('has-background-image');
      this.terminalContainer.querySelector('.terminal-background-image')?.remove();
    }

    const backgroundImageOpacity = options?.backgroundImageOpacity ?? 0.5;
    const overlayOpacity = canUseBackgroundImage
      ? clamp(1 - backgroundImageOpacity, 0, 1)
      : 0;
    const blurAmount = options?.blurAmount ?? 0;
    const blurEnabled = canUseBackgroundImage && !!options?.enableBlur && blurAmount > 0;

    this.applyAppearanceStyleRule({
      backgroundImage: canUseBackgroundImage ? toCssUrl(options?.backgroundImage) : 'none',
      overlayOpacity,
      backgroundSize: normalizeBackgroundSize(options?.backgroundImageSize),
      backgroundPosition: normalizeBackgroundPosition(options?.backgroundImagePosition),
      blur: blurEnabled ? `${blurAmount}px` : '0px',
      scale: blurEnabled ? '1.05' : '1',
      textOpacity: canUseBackgroundImage ? String(options?.textOpacity ?? 1.0) : '1',
      backgroundColor: canUseBackgroundImage
        ? 'transparent'
        : this.terminalInstance.getEffectiveBackgroundColor(),
    });
  }

  private attachTerminalToContainer(options: TerminalAttachOptions = {}): void {
    if (!this.terminalContainer || !this.terminalInstance) {
      errorLog('[TerminalView] Render failed: missing container or instance');
      return;
    }

    const bgLayer = this.terminalContainer.querySelector('.terminal-background-image');
    const dropHint = this.dropHintEl;
    this.terminalContainer.empty();
    if (bgLayer) this.terminalContainer.appendChild(bgLayer);
    if (dropHint) this.terminalContainer.appendChild(dropHint);

    try {
      this.terminalInstance.attachToElement(this.terminalContainer);
    } catch (error) {
      errorLog('[TerminalView] Attach failed:', error);
      new Notice(t('notices.terminal.renderFailed', { message: String(error) }));
      return;
    }

    window.setTimeout(() => {
      if (this.terminalInstance?.isAlive()) {
        this.terminalInstance.fit();
        if (options.focus !== false) {
          this.terminalInstance.focus();
        }
      }
    }, 100);
  }

  private setupResizeObserver(): void {
    if (!this.terminalContainer) return;
    this.resizeObserver?.disconnect();

    let resizeTimeout: number | null = null;
    const ResizeObserverCtor = this.terminalContainer.ownerDocument.defaultView?.ResizeObserver ?? ResizeObserver;

    this.resizeObserver = new ResizeObserverCtor((entries) => {
      if (resizeTimeout) window.clearTimeout(resizeTimeout);

      resizeTimeout = window.setTimeout(() => {
        if (this.terminalInstance?.isAlive()) {
          const { width, height } = entries[0].contentRect;
          if (width > 0 && height > 0) {
            this.terminalInstance.fit();
          }
        }
      }, 100);
    });

    this.resizeObserver.observe(this.terminalContainer);
  }

  /**
   * Refresh theme/background-related appearance
   */
  refreshAppearance(): void {
    if (!this.terminalInstance) return;

    const plugin = this.getTerminalPlugin();
    if (!plugin) return;

    const settings = plugin.settings;

    this.terminalInstance.updateOptions({
      fontSize: settings.fontSize,
      fontFamily: settings.fontFamily,
      cursorStyle: settings.cursorStyle,
      cursorBlink: settings.cursorBlink,
      useObsidianTheme: settings.useObsidianTheme,
      backgroundColor: settings.backgroundColor,
      foregroundColor: settings.foregroundColor,
      backgroundImage: settings.backgroundImage,
      backgroundImageOpacity: settings.backgroundImageOpacity,
      backgroundImageSize: settings.backgroundImageSize,
      backgroundImagePosition: settings.backgroundImagePosition,
      enableBlur: settings.enableBlur,
      blurAmount: settings.blurAmount,
      textOpacity: settings.textOpacity,
      preferredRenderer: settings.preferredRenderer,
    });

    this.updateAppearanceStyles();
  }

  private ensureBackgroundLayer(): void {
    if (!this.terminalContainer) return;
    const existingLayer = this.terminalContainer.querySelector('.terminal-background-image');
    if (existingLayer) return;

    const bgLayer = activeDocument.createElement('div');
    bgLayer.className = 'terminal-background-image';
    this.terminalContainer.prepend(bgLayer);
  }

  private applyAppearanceStyleRule(vars: {
    backgroundImage: string;
    overlayOpacity: number;
    backgroundSize: string;
    backgroundPosition: string;
    blur: string;
    scale: string;
    textOpacity: string;
    backgroundColor: string;
  }): void {
    if (!this.terminalContainer) return;
    const style = this.terminalContainer.style;
    style.setProperty('--terminal-bg-image', vars.backgroundImage);
    style.setProperty('--terminal-bg-overlay-opacity', String(vars.overlayOpacity));
    style.setProperty('--terminal-bg-size', vars.backgroundSize);
    style.setProperty('--terminal-bg-position', vars.backgroundPosition);
    style.setProperty('--terminal-bg-blur', vars.blur);
    style.setProperty('--terminal-bg-scale', vars.scale);
    style.setProperty('--terminal-text-opacity', vars.textOpacity);
    style.setProperty('--terminal-bg-color', vars.backgroundColor);
    const viewContainer = this.containerEl.querySelector<HTMLElement>('.terminal-view-container');
    viewContainer?.style.setProperty('--terminal-bg-color', vars.backgroundColor);
  }

  private disposeAppearanceStyle(): void {
    if (!this.terminalContainer) return;
    const style = this.terminalContainer.style;
    style.removeProperty('--terminal-bg-image');
    style.removeProperty('--terminal-bg-overlay-opacity');
    style.removeProperty('--terminal-bg-size');
    style.removeProperty('--terminal-bg-position');
    style.removeProperty('--terminal-bg-blur');
    style.removeProperty('--terminal-bg-scale');
    style.removeProperty('--terminal-text-opacity');
    style.removeProperty('--terminal-bg-color');
    const viewContainer = this.containerEl.querySelector<HTMLElement>('.terminal-view-container');
    viewContainer?.style.removeProperty('--terminal-bg-color');
  }

  /**
   * Get the terminal instance (for external callers)
   */
  getTerminalInstance(): TerminalInstance | null {
    return this.terminalInstance;
  }

  async waitForTerminalInstance(timeoutMs = 8000): Promise<TerminalInstance> {
    if (this.terminalInstance) return this.terminalInstance;
    if (!this.initPromise) {
      throw new Error(t('terminal.notInitialized'));
    }

    const timeoutPromise = new Promise<never>((_, reject) => {
      window.setTimeout(() => reject(new Error(t('terminal.notInitialized'))), timeoutMs);
    });

    return Promise.race([this.initPromise, timeoutPromise]);
  }

  private updateLeafHeader(leaf: WorkspaceLeaf): void {
    const leafWithHeader = leaf as WorkspaceLeaf & { updateHeader?: () => void };
    leafWithHeader.updateHeader?.();
  }

  private getTerminalPlugin(): {
    settings: TerminalSettings;
    activateTerminalView: () => Promise<void>;
    reconnectTerminalView: (terminalView: TerminalView) => Promise<void>;
    isRemoteTerminal: (terminal: TerminalInstance) => boolean;
    getRemoteNodeId: (terminal: TerminalInstance) => string | null;
    getDeviceConnectionManager: () => DeviceConnectionManager;
    toggleAlwaysOnTopTerminal: (terminalView: TerminalView) => Promise<void>;
    getAlwaysOnTopTerminalLabel: (terminalView: TerminalView) => string;
    isAlwaysOnTopTerminal: (terminalView: TerminalView) => boolean;
    handleTerminalViewClosed: (terminalView: TerminalView) => void;
    saveSettings: () => Promise<void>;
    copyDirectoryTreeEntryToVaultWithPicker: (entry: DirectoryTreeDragPayload, explicitTargetFolder?: string) => Promise<void>;
  } | null {
    const appWithPlugins = this.app as typeof this.app & {
      plugins?: { getPlugin?: (id: string) => unknown };
    };
    const plugin = appWithPlugins.plugins?.getPlugin?.('termesh');
    if (!this.isTerminalPlugin(plugin)) return null;
    return plugin;
  }

  private isTerminalPlugin(value: unknown): value is {
    settings: TerminalSettings;
    activateTerminalView: () => Promise<void>;
    reconnectTerminalView: (terminalView: TerminalView) => Promise<void>;
    isRemoteTerminal: (terminal: TerminalInstance) => boolean;
    getRemoteNodeId: (terminal: TerminalInstance) => string | null;
    getDeviceConnectionManager: () => DeviceConnectionManager;
    toggleAlwaysOnTopTerminal: (terminalView: TerminalView) => Promise<void>;
    getAlwaysOnTopTerminalLabel: (terminalView: TerminalView) => string;
    isAlwaysOnTopTerminal: (terminalView: TerminalView) => boolean;
    handleTerminalViewClosed: (terminalView: TerminalView) => void;
    saveSettings: () => Promise<void>;
    copyDirectoryTreeEntryToVaultWithPicker: (entry: DirectoryTreeDragPayload, explicitTargetFolder?: string) => Promise<void>;
  } {
    if (!value || typeof value !== 'object') return false;
    const candidate = value as {
      settings?: unknown;
      activateTerminalView?: unknown;
      reconnectTerminalView?: unknown;
      isRemoteTerminal?: unknown;
      getRemoteNodeId?: unknown;
      getDeviceConnectionManager?: unknown;
      toggleAlwaysOnTopTerminal?: unknown;
      getAlwaysOnTopTerminalLabel?: unknown;
      isAlwaysOnTopTerminal?: unknown;
      handleTerminalViewClosed?: unknown;
      saveSettings?: unknown;
      copyDirectoryTreeEntryToVaultWithPicker?: unknown;
    };
    return typeof candidate.activateTerminalView === 'function'
      && typeof candidate.reconnectTerminalView === 'function'
      && typeof candidate.isRemoteTerminal === 'function'
      && typeof candidate.getRemoteNodeId === 'function'
      && typeof candidate.getDeviceConnectionManager === 'function'
      && typeof candidate.toggleAlwaysOnTopTerminal === 'function'
      && typeof candidate.getAlwaysOnTopTerminalLabel === 'function'
      && typeof candidate.isAlwaysOnTopTerminal === 'function'
      && typeof candidate.handleTerminalViewClosed === 'function'
      && typeof candidate.saveSettings === 'function'
      && typeof candidate.copyDirectoryTreeEntryToVaultWithPicker === 'function'
      && typeof candidate.settings === 'object';
  }
}
