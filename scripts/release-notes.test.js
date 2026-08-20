import test from 'node:test';
import assert from 'node:assert/strict';
import { extractChangelogSection, renderReleaseBody } from './release-notes.js';

const SAMPLE_CHANGELOG = `# Changelog

## [1.3.0] - 2026-04-23

### Added
- Embedded changelog support.

## [1.2.3]

### Fixed
- Previous release.
`;

test('extractChangelogSection returns the requested version notes', () => {
  const section = extractChangelogSection(SAMPLE_CHANGELOG, '1.3.0');

  assert.equal(section, [
    '### Added',
    '- Embedded changelog support.',
  ].join('\n'));
});

test('renderReleaseBody describes the package without requiring CHANGELOG.md as an asset', () => {
  const body = renderReleaseBody({
    version: '1.3.0',
    changelogSection: '### Added\n- Embedded changelog support.',
    repository: 'jiang-zhong-xi/Termy',
  });

  assert.match(body, /Download the `termesh-1\.3\.0-<platform>\.zip` asset matching your platform/);
  assert.doesNotMatch(body, /Download `termesh\.zip`/);
  assert.doesNotMatch(body, /includes all platform binaries and `CHANGELOG\.md`/);
  assert.match(body, /automatically downloads and verifies the platform-native runtime/);
  assert.match(body, /Complete Package \(Offline Installation\)/);
  assert.match(body, /\[Telegram Group\]\(https:\/\/t\.me\/\+t6oRqhaw8c1jNzE1\)/);
  assert.doesNotMatch(body, /\[Discussions\]\(/);
  assert.ok(body.startsWith('## Changelog\n\n### Added\n- Embedded changelog support.\n\n## Installation'));

  // Community plugins is the recommended path; BRAT is offered as the early-updates fallback.
  const communityIndex = body.indexOf('### Obsidian Community Plugins (Recommended)');
  const bratIndex = body.indexOf('### BRAT (Early Updates)');
  const manualIndex = body.indexOf('### Manual Installation');
  assert.ok(communityIndex !== -1, 'Community Plugins section should be present');
  assert.ok(bratIndex !== -1, 'BRAT section should be present');
  assert.ok(manualIndex !== -1, 'Manual Installation section should be present');
  assert.ok(communityIndex < bratIndex, 'Community Plugins should come before BRAT');
  assert.ok(bratIndex < manualIndex, 'BRAT should come before Manual Installation');
  assert.match(body, /Click \*\*Browse\*\*, search for `Termesh`/);
  assert.match(body, /`termesh-agent-win32-x64\.exe`: Windows x64 remote agent/);
  assert.match(body, /`termesh-agent-linux-x64`: Linux x64 remote agent/);
  assert.match(body, /`termy-relay-linux-x64`: Linux x64 relay server/);
});
