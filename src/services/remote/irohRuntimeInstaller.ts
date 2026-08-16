import crypto from 'node:crypto';
import fs from 'node:fs';
import https from 'node:https';
import path from 'node:path';

const MAX_REDIRECTS = 5;
const DOWNLOAD_IDLE_TIMEOUT_MS = 15_000;
const GITHUB_RELEASE_REPOSITORY = 'jiang-zhong-xi/Termy';
const IROH_VERSION = '1.1.0';

const RUNTIME_ASSETS: Record<string, {
  packageName: string;
  packageFile: string;
  sha256: string;
}> = {
  'darwin-arm64': {
    packageName: '@number0/iroh-darwin-arm64',
    packageFile: 'iroh.darwin-arm64.node',
    sha256: '3d8b1c05a9994e9d36b2693e3af832ed9cd113d64ac0eaf4d7d082f2a5d1aef8',
  },
  'linux-arm64': {
    packageName: '@number0/iroh-linux-arm64-gnu',
    packageFile: 'iroh.linux-arm64-gnu.node',
    sha256: 'e19d9da4111f5e35b630317c281b575e81c7f0abf6e3cd09dd1ba15d074eec82',
  },
  'linux-x64': {
    packageName: '@number0/iroh-linux-x64-gnu',
    packageFile: 'iroh.linux-x64-gnu.node',
    sha256: 'd660aa1abbc2b65b99a970e8da094339256725078b75ed1201b49e0b0acbda55',
  },
  'win32-x64': {
    packageName: '@number0/iroh-win32-x64-msvc',
    packageFile: 'iroh.win32-x64-msvc.node',
    sha256: '4fef61d33fc9a903a21cb1a2ae154b1e9576e8e453343b295d7d4d387e44bbe1',
  },
};

export interface IrohRuntimePaths {
  nativePath: string;
}

export interface IrohRuntimeInstallProgress {
  stage: 'downloading' | 'retrying' | 'verifying' | 'complete' | 'error';
  percent?: number;
}

export interface IrohRuntimeAssetUrls {
  nativeUrl: string;
  fallbackUrls: string[];
  expectedHash: string;
}

export interface ResolveIrohRuntimeAssetUrlsOptions {
  version: string;
  platform?: NodeJS.Platform;
  arch?: string;
}

type DownloadProgress = (downloadedBytes: number, totalBytes: number) => void;
export type IrohRuntimeAssetFetcher = (url: string, onProgress?: DownloadProgress) => Promise<Buffer>;
export type IrohRuntimeAssetResolver = (
  options: ResolveIrohRuntimeAssetUrlsOptions,
) => IrohRuntimeAssetUrls;

export function resolveIrohRuntimeAssetUrls(
  options: ResolveIrohRuntimeAssetUrlsOptions,
): IrohRuntimeAssetUrls {
  const platform = options.platform ?? process.platform;
  const arch = options.arch ?? process.arch;
  const platformKey = `${platform}-${arch}`;
  const runtimeAsset = RUNTIME_ASSETS[platformKey];
  if (!runtimeAsset) {
    throw new Error(`unsupported iroh runtime platform: ${platformKey}`);
  }
  const assetBase = `iroh-runtime-${platform}-${arch}`;
  const releaseBaseUrl = `https://github.com/${GITHUB_RELEASE_REPOSITORY}/releases/download/${options.version}`;
  const npmPackagePath = `${runtimeAsset.packageName}@${IROH_VERSION}/${runtimeAsset.packageFile}`;

  return {
    nativeUrl: `https://unpkg.com/${npmPackagePath}`,
    fallbackUrls: [
      `https://cdn.jsdelivr.net/npm/${npmPackagePath}`,
      `${releaseBaseUrl}/${assetBase}.node`,
    ],
    expectedHash: runtimeAsset.sha256,
  };
}

export class IrohRuntimeInstaller {
  private readonly pluginDir: string;
  private readonly pluginVersion: string;
  private readonly fetchAsset: IrohRuntimeAssetFetcher;
  private readonly fallbackFetchAsset?: IrohRuntimeAssetFetcher;
  private readonly resolveAsset: IrohRuntimeAssetResolver;
  private readonly runtimeDir: string;
  private readonly markerPath: string;
  private useFallback = false;

  constructor(
    pluginDir: string,
    pluginVersion: string,
    fetchAsset: IrohRuntimeAssetFetcher = downloadAsset,
    fallbackFetchAsset?: IrohRuntimeAssetFetcher,
    resolveAsset: IrohRuntimeAssetResolver = resolveIrohRuntimeAssetUrls,
  ) {
    this.pluginDir = pluginDir;
    this.pluginVersion = pluginVersion;
    this.fetchAsset = fetchAsset;
    this.fallbackFetchAsset = fallbackFetchAsset;
    this.resolveAsset = resolveAsset;
    this.runtimeDir = path.join(this.pluginDir, 'native', 'iroh');
    this.markerPath = path.join(this.runtimeDir, 'version.json');
  }

  async ensureInstalled(
    onProgress?: (progress: IrohRuntimeInstallProgress) => void,
  ): Promise<IrohRuntimePaths> {
    const runtimePaths = this.getRuntimePaths();
    if (this.isInstalled(runtimePaths)) {
      return runtimePaths;
    }

    try {
      const urls = this.resolveAsset({ version: this.pluginVersion });
      onProgress?.({ stage: 'downloading', percent: 0 });
      const native = await this.fetchWithFallback(urls.nativeUrl, urls.fallbackUrls, (downloadedBytes, totalBytes) => {
        if (totalBytes > 0) {
          onProgress?.({
            stage: 'downloading',
            percent: Math.min(100, (downloadedBytes / totalBytes) * 100),
          });
        }
      }, onProgress);

      onProgress?.({ stage: 'verifying' });
  verifyChecksum(native, urls.expectedHash, 'iroh native runtime');

      fs.mkdirSync(this.runtimeDir, { recursive: true });
      const nativeTempPath = `${runtimePaths.nativePath}.download`;

      try {
        fs.writeFileSync(nativeTempPath, native);
        replaceFile(nativeTempPath, runtimePaths.nativePath);
        fs.writeFileSync(this.markerPath, JSON.stringify({ version: this.pluginVersion }));
      } catch (error) {
        safeUnlink(nativeTempPath);
        throw error;
      }

      onProgress?.({ stage: 'complete' });
      return runtimePaths;
    } catch (error) {
      onProgress?.({ stage: 'error' });
      throw error;
    }
  }

  private getRuntimePaths(): IrohRuntimePaths {
    return {
      nativePath: path.join(this.runtimeDir, 'iroh-runtime.node'),
    };
  }

  private async fetchWithFallback(
    url: string,
    fallbackUrls: string[],
    onDownloadProgress: DownloadProgress | undefined,
    onInstallProgress: ((progress: IrohRuntimeInstallProgress) => void) | undefined,
  ): Promise<Buffer> {
    const urls = [url, ...fallbackUrls];
    let lastError: unknown;

    if (!this.useFallback) {
      for (const assetUrl of urls) {
        try {
          return await this.fetchAsset(assetUrl, onDownloadProgress);
        } catch (error) {
          lastError = error;
        }
      }

      if (!this.fallbackFetchAsset) throw lastError;
      this.useFallback = true;
      onInstallProgress?.({ stage: 'retrying' });
    }

    for (const assetUrl of urls) {
      try {
        return await this.fallbackFetchAsset!(assetUrl, onDownloadProgress);
      } catch (error) {
        lastError = error;
      }
    }
    throw lastError;
  }

  private isInstalled(runtimePaths: IrohRuntimePaths): boolean {
    if (!fs.existsSync(runtimePaths.nativePath)) {
      return false;
    }

    try {
      const marker = JSON.parse(fs.readFileSync(this.markerPath, 'utf8')) as { version?: string };
      return marker.version === this.pluginVersion;
    } catch {
      return false;
    }
  }
}

function verifyChecksum(content: Buffer, expectedHash: string, label: string): void {
  if (!expectedHash || !/^[a-f0-9]{64}$/.test(expectedHash)) {
    throw new Error(`${label} checksum is invalid`);
  }

  const actualHash = crypto.createHash('sha256').update(content).digest('hex');
  if (actualHash !== expectedHash) {
    throw new Error(`${label} checksum mismatch`);
  }
}

function replaceFile(sourcePath: string, destinationPath: string): void {
  if (fs.existsSync(destinationPath)) {
    fs.unlinkSync(destinationPath);
  }
  fs.renameSync(sourcePath, destinationPath);
}

function safeUnlink(filePath: string): void {
  try {
    fs.unlinkSync(filePath);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
      throw error;
    }
  }
}

async function downloadAsset(url: string, onProgress?: DownloadProgress): Promise<Buffer> {
  return downloadAssetWithRedirects(url, MAX_REDIRECTS, onProgress);
}

async function downloadAssetWithRedirects(
  url: string,
  remainingRedirects: number,
  onProgress?: DownloadProgress,
): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const request = https.get(url, { headers: { 'User-Agent': 'Termesh' } }, (response) => {
      const statusCode = response.statusCode ?? 0;
      const location = response.headers.location;

      if (statusCode >= 300 && statusCode < 400 && location) {
        response.resume();
        if (remainingRedirects === 0) {
          reject(new Error('iroh runtime download exceeded the redirect limit'));
          return;
        }
        const redirectUrl = new URL(location, url).toString();
        resolve(downloadAssetWithRedirects(redirectUrl, remainingRedirects - 1, onProgress));
        return;
      }

      if (statusCode !== 200) {
        response.resume();
        reject(new Error(`iroh runtime download failed: HTTP ${statusCode}`));
        return;
      }

      const chunks: Buffer[] = [];
      const totalBytes = Number(response.headers['content-length'] ?? 0);
      let downloadedBytes = 0;
      response.on('data', (chunk: Buffer) => {
        chunks.push(chunk);
        downloadedBytes += chunk.length;
        onProgress?.(downloadedBytes, totalBytes);
      });
      response.on('end', () => resolve(Buffer.concat(chunks)));
      response.on('error', reject);
    });

    request.setTimeout(DOWNLOAD_IDLE_TIMEOUT_MS, () => {
      request.destroy(new Error('iroh runtime download timed out'));
    });
    request.on('error', reject);
  });
}