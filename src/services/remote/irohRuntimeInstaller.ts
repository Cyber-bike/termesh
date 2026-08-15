import crypto from 'node:crypto';
import fs from 'node:fs';
import https from 'node:https';
import path from 'node:path';

const MAX_REDIRECTS = 5;
const GITHUB_RELEASE_REPOSITORY = 'jiang-zhong-xi/Termy';

export interface IrohRuntimePaths {
  nativePath: string;
}

export interface IrohRuntimeAssetUrls {
  nativeUrl: string;
  nativeChecksumUrl: string;
}

interface ResolveIrohRuntimeAssetUrlsOptions {
  version: string;
  platform?: NodeJS.Platform;
  arch?: string;
}

type FetchAsset = (url: string) => Promise<Buffer>;

export function resolveIrohRuntimeAssetUrls(
  options: ResolveIrohRuntimeAssetUrlsOptions,
): IrohRuntimeAssetUrls {
  const platform = options.platform ?? process.platform;
  const arch = options.arch ?? process.arch;
  const assetBase = `iroh-runtime-${platform}-${arch}`;
  const releaseBaseUrl = `https://github.com/${GITHUB_RELEASE_REPOSITORY}/releases/download/${options.version}`;

  return {
    nativeUrl: `${releaseBaseUrl}/${assetBase}.node`,
    nativeChecksumUrl: `${releaseBaseUrl}/${assetBase}.node.sha256`,
  };
}

export class IrohRuntimeInstaller {
  private readonly pluginDir: string;
  private readonly pluginVersion: string;
  private readonly fetchAsset: FetchAsset;
  private readonly runtimeDir: string;
  private readonly markerPath: string;

  constructor(
    pluginDir: string,
    pluginVersion: string,
    fetchAsset: FetchAsset = downloadAsset,
  ) {
    this.pluginDir = pluginDir;
    this.pluginVersion = pluginVersion;
    this.fetchAsset = fetchAsset;
    this.runtimeDir = path.join(this.pluginDir, 'native', 'iroh');
    this.markerPath = path.join(this.runtimeDir, 'version.json');
  }

  async ensureInstalled(): Promise<IrohRuntimePaths> {
    const runtimePaths = this.getRuntimePaths();
    if (this.isInstalled(runtimePaths)) {
      return runtimePaths;
    }

    const urls = resolveIrohRuntimeAssetUrls({ version: this.pluginVersion });
    const [native, nativeChecksum] = await Promise.all([
      this.fetchAsset(urls.nativeUrl),
      this.fetchAsset(urls.nativeChecksumUrl),
    ]);

    verifyChecksum(native, nativeChecksum.toString('utf8'), 'iroh native runtime');

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

    return runtimePaths;
  }

  private getRuntimePaths(): IrohRuntimePaths {
    return {
      nativePath: path.join(this.runtimeDir, 'iroh-runtime.node'),
    };
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

function verifyChecksum(content: Buffer, checksumFile: string, label: string): void {
  const expectedHash = checksumFile.trim().split(/\s+/)[0]?.toLowerCase();
  if (!expectedHash || !/^[a-f0-9]{64}$/.test(expectedHash)) {
    throw new Error(`${label} checksum file is invalid`);
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

async function downloadAsset(url: string): Promise<Buffer> {
  return downloadAssetWithRedirects(url, MAX_REDIRECTS);
}

async function downloadAssetWithRedirects(url: string, remainingRedirects: number): Promise<Buffer> {
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
        resolve(downloadAssetWithRedirects(redirectUrl, remainingRedirects - 1));
        return;
      }

      if (statusCode !== 200) {
        response.resume();
        reject(new Error(`iroh runtime download failed: HTTP ${statusCode}`));
        return;
      }

      const chunks: Buffer[] = [];
      response.on('data', (chunk: Buffer) => chunks.push(chunk));
      response.on('end', () => resolve(Buffer.concat(chunks)));
      response.on('error', reject);
    });

    request.on('error', reject);
  });
}