import path from 'node:path';

import { IrohRuntimeInstaller } from './irohRuntimeInstaller';
import type {
  IrohRuntimeAssetFetcher,
  IrohRuntimeInstallProgress,
  IrohRuntimePaths,
} from './irohRuntimeInstaller';
import type { IrohModule } from './irohStreams';

type ModuleLoader = (modulePath: string) => unknown;

interface IrohLoaderOptions {
  version: string;
  isOffline: () => boolean;
  installRuntime?: (
    onProgress?: (progress: IrohRuntimeInstallProgress) => void,
  ) => Promise<IrohRuntimePaths>;
  fallbackFetchAsset?: IrohRuntimeAssetFetcher;
  onInstallProgress?: (progress: IrohRuntimeInstallProgress) => void;
}

export function createIrohLoader(
  pluginDir: string,
  loadModule: ModuleLoader = require,
  options?: IrohLoaderOptions,
): () => Promise<IrohModule> {
  const modulePath = path.join(pluginDir, 'node_modules', '@number0', 'iroh');
  const installer = options
    ? new IrohRuntimeInstaller(pluginDir, options.version, undefined, options.fallbackFetchAsset)
    : null;
  const installRuntime = options?.installRuntime ?? ((onProgress) => installer!.ensureInstalled(onProgress));
  let loadedModule: IrohModule | null = null;
  let loading: Promise<IrohModule> | null = null;

  return () => {
    if (loadedModule) {
      return Promise.resolve(loadedModule);
    }

    try {
      loadedModule = loadModule(modulePath) as IrohModule;
      return Promise.resolve(loadedModule);
    } catch (error) {
      if (!options) {
        return Promise.reject(createLoadError(error));
      }
      if (options.isOffline()) {
        return Promise.reject(
          new Error('无法加载远程终端原生模块：离线模式禁止自动下载。请关闭离线模式后重试，或安装平台完整包。', {
            cause: error,
          }),
        );
      }
      if (loading) {
        return loading;
      }

      loading = installRuntime((progress) => {
        if (progress.stage !== 'complete' && progress.stage !== 'error') {
          options.onInstallProgress?.(progress);
        }
      })
        .then((runtimePaths) => loadModule(runtimePaths.nativePath) as IrohModule)
        .then((module) => {
          loadedModule = module;
          options.onInstallProgress?.({ stage: 'complete' });
          return module;
        })
        .catch((installError: unknown) => {
          options.onInstallProgress?.({ stage: 'error' });
          throw createLoadError(installError);
        })
        .finally(() => {
          loading = null;
        });
      return loading;
    }
  };
}

function createLoadError(error: unknown): Error {
  const detail = error instanceof Error ? error.message : String(error);
  return new Error(
    `无法加载远程终端原生模块。请检查网络后重试；离线环境请安装 Termesh 平台完整包。开发安装请先运行 pnpm package。详情：${detail}`,
    { cause: error },
  );
}