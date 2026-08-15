import fs from 'node:fs';
import path from 'node:path';

export function copyPackagedRuntime(packageDir, targetDir) {
  const entries = ['main.js', 'manifest.json', 'styles.css', 'binaries', 'node_modules'];

  for (const entry of entries) {
    const sourcePath = path.join(packageDir, entry);
    if (!fs.existsSync(sourcePath)) {
      throw new Error(`Packaged runtime entry is missing: ${entry}`);
    }

    const targetPath = path.join(targetDir, entry);
    if (fs.statSync(sourcePath).isDirectory()) {
      fs.rmSync(targetPath, { recursive: true, force: true });
      fs.cpSync(sourcePath, targetPath, { recursive: true, dereference: true });
    } else {
      fs.copyFileSync(sourcePath, targetPath);
    }
  }
}

export function migrateEnabledPluginId(pluginsDir, legacyId = 'termesh', pluginId = 'termy') {
  const enabledPluginsPath = path.join(path.dirname(pluginsDir), 'community-plugins.json');
  if (!fs.existsSync(enabledPluginsPath)) {
    return false;
  }

  const enabledPlugins = JSON.parse(fs.readFileSync(enabledPluginsPath, 'utf8'));
  if (!Array.isArray(enabledPlugins) || !enabledPlugins.includes(legacyId)) {
    return false;
  }

  const migratedPlugins = enabledPlugins
    .map((id) => id === legacyId ? pluginId : id)
    .filter((id, index, ids) => ids.indexOf(id) === index);
  fs.writeFileSync(enabledPluginsPath, `${JSON.stringify(migratedPlugins, null, 2)}\n`);
  return true;
}

export function migrateLegacyPluginData(pluginsDir, legacyId = 'termesh', pluginId = 'termy') {
  const legacyDataPath = path.join(pluginsDir, legacyId, 'data.json');
  const targetDataPath = path.join(pluginsDir, pluginId, 'data.json');
  if (!fs.existsSync(legacyDataPath) || fs.existsSync(targetDataPath)) {
    return false;
  }

  fs.mkdirSync(path.dirname(targetDataPath), { recursive: true });
  fs.copyFileSync(legacyDataPath, targetDataPath);
  return true;
}