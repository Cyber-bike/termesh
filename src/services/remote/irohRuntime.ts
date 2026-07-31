import path from 'node:path';

import type { IrohModule } from './irohStreams';

type ModuleLoader = (modulePath: string) => unknown;

export function createIrohLoader(
  pluginDir: string,
  loadModule: ModuleLoader = require,
): () => Promise<IrohModule> {
  const modulePath = path.join(pluginDir, 'node_modules', '@number0', 'iroh');

  return () => {
    try {
      return Promise.resolve(loadModule(modulePath) as IrohModule);
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      return Promise.reject(
        new Error(
          `无法加载远程终端原生模块。请使用 Termy 完整安装包重新安装插件；开发安装请先运行 pnpm package。详情：${detail}`,
          { cause: error },
        ),
      );
    }
  };
}