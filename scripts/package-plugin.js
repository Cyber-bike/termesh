/**
 * Plugin Package Script
 * Package plugin files for distribution
 *
 * Usage:
 *   node scripts/package-plugin.js        # Package for current platform
 *   node scripts/package-plugin.js --zip  # Create ZIP archive
 */

import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { createRequire } from 'module';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const require = createRequire(import.meta.url);

// Termy server configuration
const SERVER_CONFIG = {
  name: 'termy-server',
  displayName: 'Termy Server'
};

// Parse command line arguments
const args = process.argv.slice(2);
const createZip = args.includes('--zip');

// Detect current platform
function getCurrentPlatform() {
  const platform = process.platform;
  const arch = process.arch;
  return `${platform}-${arch}`;
}

const currentPlatform = getCurrentPlatform();

console.log('📦 Plugin Package Script');
console.log(`🔍 Current platform: ${currentPlatform}`);
console.log('');

// Project paths
const ROOT_DIR = path.join(__dirname, '..');
const BINARIES_DIR = path.join(ROOT_DIR, 'binaries');
const PACKAGE_DIR = path.join(ROOT_DIR, 'plugin-package');

// 1. Check required files
console.log('🔍 Checking required files...');
const coreFiles = [
  'main.js',
  'manifest.json',
  'styles.css'
];
const packageFiles = [...coreFiles];

for (const file of packageFiles) {
  const filePath = path.join(ROOT_DIR, file);
  if (!fs.existsSync(filePath)) {
    console.error(`❌ Error: Missing required file ${file}`);
    console.error('Please run pnpm build first and ensure the plugin bundle files are present');
    process.exit(1);
  }
}
console.log('✅ All required files exist');
console.log('');

// 2. Check binary file for current platform
console.log('🔍 Checking binary file...');

const ext = currentPlatform.startsWith('win32') ? '.exe' : '';
const binaryName = `${SERVER_CONFIG.name}-${currentPlatform}${ext}`;
const binaryPath = path.join(BINARIES_DIR, binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error(`  ❌ Missing: ${binaryName}`);
  console.error('');
  console.error('Please run: node scripts/build-rust.js');
  process.exit(1);
}

const binaryStats = fs.statSync(binaryPath);
const binarySizeMB = (binaryStats.size / 1024 / 1024).toFixed(2);
console.log(`  ✓ ${binaryName} (${binarySizeMB} MB)`);
console.log('');

// 3. Clean and create package directory
if (fs.existsSync(PACKAGE_DIR)) {
  fs.rmSync(PACKAGE_DIR, { recursive: true, force: true });
}
fs.mkdirSync(PACKAGE_DIR, { recursive: true });
fs.mkdirSync(path.join(PACKAGE_DIR, 'binaries'), { recursive: true });

console.log('📋 Copying files to package directory...');

// 4. Copy package files
for (const file of packageFiles) {
  const srcPath = path.join(ROOT_DIR, file);
  const destPath = path.join(PACKAGE_DIR, file);
  fs.copyFileSync(srcPath, destPath);
  console.log(`  ✓ ${file}`);
}

// 5. Copy binary file
const destBinaryPath = path.join(PACKAGE_DIR, 'binaries', binaryName);
fs.copyFileSync(binaryPath, destBinaryPath);

console.log(`  ✓ binaries/${binaryName}`);
console.log('');

// 5b. Copy the @number0/iroh native module (v2.0 remote terminal, A0
// direct-embedding path). esbuild marks it `external` (see
// esbuild.config.mjs), so main.js does a bare `require('@number0/iroh')` at
// runtime - Obsidian plugin folders are not on any shared node_modules
// resolution path, so the module has to physically travel inside the
// package, next to main.js.
//
// The platform-native package (`@number0/iroh-<platform>`) is an
// *optionalDependency* of `@number0/iroh`, resolved and installed only for
// the machine `pnpm install` ran on - never hardcode a platform->package
// name mapping here, since that list has already changed once (darwin-x64
// was dropped between npm registry snapshots during A0 investigation).
// Instead read the real optionalDependencies from the installed package and
// let Node's own resolver find which one actually landed, exactly like
// `require('@number0/iroh')` will at runtime. pnpm's isolated store means
// the resolved paths are typically symlinks into node_modules/.pnpm/, so
// every copy below dereferences them into real files.
console.log('🔍 Locating @number0/iroh native module...');

function resolvePackageDir(specifier, fromDir) {
  const pkgJsonPath = require.resolve(`${specifier}/package.json`, {
    paths: [fromDir],
  });
  return path.dirname(pkgJsonPath);
}

let irohDir;
try {
  irohDir = resolvePackageDir('@number0/iroh', ROOT_DIR);
} catch {
  console.error('❌ Error: @number0/iroh is not installed.');
  console.error('Please run `pnpm install` first (it is a regular dependency in package.json).');
  process.exit(1);
}

const irohPkg = JSON.parse(fs.readFileSync(path.join(irohDir, 'package.json'), 'utf8'));
const candidatePlatformPackages = Object.keys(irohPkg.optionalDependencies ?? {});

let platformPackageName = null;
let platformPackageDir = null;
for (const candidate of candidatePlatformPackages) {
  try {
    platformPackageDir = resolvePackageDir(candidate, irohDir);
    platformPackageName = candidate;
    break;
  } catch {
    // Not the platform this install is for - every other candidate is
    // expected to fail resolution, that's how npm's os/cpu gating works.
  }
}

if (!platformPackageDir) {
  console.error('❌ Error: no @number0/iroh-<platform> native package resolved for this machine.');
  console.error(`Checked: ${candidatePlatformPackages.join(', ')}`);
  console.error('Re-run `pnpm install` on the target platform, then package there.');
  process.exit(1);
}

const pluginNodeModulesDir = path.join(PACKAGE_DIR, 'node_modules', '@number0');
fs.mkdirSync(pluginNodeModulesDir, { recursive: true });

const destIrohDir = path.join(pluginNodeModulesDir, 'iroh');
const destPlatformDir = path.join(pluginNodeModulesDir, platformPackageName.replace('@number0/', ''));
fs.cpSync(irohDir, destIrohDir, { recursive: true, dereference: true });
fs.cpSync(platformPackageDir, destPlatformDir, { recursive: true, dereference: true });

function dirSize(dir) {
  let total = 0;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const entryPath = path.join(dir, entry.name);
    total += entry.isDirectory() ? dirSize(entryPath) : fs.statSync(entryPath).size;
  }
  return total;
}

const irohModuleSize = dirSize(destIrohDir) + dirSize(destPlatformDir);
console.log(`  ✓ node_modules/@number0/iroh (${irohPkg.version})`);
console.log(`  ✓ node_modules/@number0/${platformPackageName.replace('@number0/', '')}`);
console.log('');

// 6. Calculate package size
console.log('📊 Package size statistics...');
let totalSize = 0;

for (const file of packageFiles) {
  const filePath = path.join(PACKAGE_DIR, file);
  const stats = fs.statSync(filePath);
  totalSize += stats.size;
  const sizeKB = (stats.size / 1024).toFixed(1);
  console.log(`  ${file}: ${sizeKB} KB`);
}

totalSize += binaryStats.size;
console.log(`  binaries/${binaryName}: ${binarySizeMB} MB`);

totalSize += irohModuleSize;
console.log(`  node_modules/@number0: ${(irohModuleSize / 1024 / 1024).toFixed(2)} MB`);

const totalSizeMB = (totalSize / 1024 / 1024).toFixed(2);
console.log(`  Total: ${totalSizeMB} MB`);
console.log('');

// 7. Create ZIP if requested
if (createZip) {
  console.log('📦 Creating ZIP archive...');
  
  // Read version from manifest
  const manifestPath = path.join(PACKAGE_DIR, 'manifest.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const version = manifest.version || '0.0.0';
  
  const zipName = `termy-${version}.zip`;
  const zipPath = path.join(ROOT_DIR, zipName);
  
  // Remove existing ZIP
  if (fs.existsSync(zipPath)) {
    fs.unlinkSync(zipPath);
  }
  
  try {
    // Use PowerShell Compress-Archive (Windows) or zip command (Unix)
    if (process.platform === 'win32') {
      execSync(
        `powershell -Command "Compress-Archive -Path '${PACKAGE_DIR}\\*' -DestinationPath '${zipPath}' -Force"`,
        { stdio: 'inherit' }
      );
    } else {
      execSync(
        `cd "${PACKAGE_DIR}" && zip -r "${zipPath}" .`,
        { stdio: 'inherit' }
      );
    }
    
    const zipStats = fs.statSync(zipPath);
    const zipSizeMB = (zipStats.size / 1024 / 1024).toFixed(2);
    console.log(`  ✅ ZIP created: ${zipName} (${zipSizeMB} MB)`);
  } catch (error) {
    console.error('  ❌ Failed to create ZIP:', error.message);
    console.log('  💡 Tip: You can manually compress the plugin-package/ directory');
  }
  
  console.log('');
}

// 8. Complete
console.log('🎉 Package complete!');
console.log(`📂 Package location: ${PACKAGE_DIR}`);

if (createZip) {
  console.log('');
  console.log('📦 Next steps:');
  console.log('  1. Test the packaged plugin in Obsidian');
  console.log('  2. Upload to GitHub Releases');
  console.log('  3. Submit to Obsidian community plugins');
}
