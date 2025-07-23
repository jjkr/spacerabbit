#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

/**
 * Checks that all version files are synchronized with package.json
 * Used in CI to ensure version consistency
 */

function main() {
  try {
    // Read version from package.json (source of truth)
    const packageJsonPath = path.join(__dirname, '..', 'package.json');
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    const expectedVersion = packageJson.version;

    if (!expectedVersion) {
      console.error('❌ No version found in package.json');
      process.exit(1);
    }

    console.log(`🔍 Checking version consistency (expected: ${expectedVersion})`);

    let allInSync = true;
    const results = [];

    // Check Cargo.toml
    const cargoResult = checkCargoToml(expectedVersion);
    results.push(cargoResult);
    if (!cargoResult.inSync) allInSync = false;

    // Check tauri.conf.json
    const tauriResult = checkTauriConfig(expectedVersion);
    results.push(tauriResult);
    if (!tauriResult.inSync) allInSync = false;

    // Check package-lock.json
    const lockResult = checkPackageLock(expectedVersion);
    results.push(lockResult);
    if (!lockResult.inSync) allInSync = false;

    // Print results
    console.log('\n📋 Version Check Results:');
    results.forEach(result => {
      const status = result.inSync ? '✅' : '❌';
      const version = result.found ? result.version : 'NOT FOUND';
      console.log(`${status} ${result.file}: ${version}`);
    });

    if (allInSync) {
      console.log('\n🎉 All versions are synchronized!');
      process.exit(0);
    } else {
      console.log('\n💡 To fix version mismatches, run: npm run version:sync');
      process.exit(1);
    }

  } catch (error) {
    console.error('❌ Error checking versions:', error.message);
    process.exit(1);
  }
}

function checkCargoToml(expectedVersion) {
  const cargoPath = path.join(__dirname, '..', 'src-tauri', 'Cargo.toml');
  
  if (!fs.existsSync(cargoPath)) {
    return { file: 'src-tauri/Cargo.toml', found: false, inSync: false };
  }

  const cargoContent = fs.readFileSync(cargoPath, 'utf8');
  const versionMatch = cargoContent.match(/^version\s*=\s*"([^"]*)"/m);
  
  if (!versionMatch) {
    return { file: 'src-tauri/Cargo.toml', found: false, inSync: false };
  }

  const version = versionMatch[1];
  return {
    file: 'src-tauri/Cargo.toml',
    found: true,
    version,
    inSync: version === expectedVersion
  };
}

function checkTauriConfig(expectedVersion) {
  const configPath = path.join(__dirname, '..', 'src-tauri', 'tauri.conf.json');
  
  if (!fs.existsSync(configPath)) {
    return { file: 'src-tauri/tauri.conf.json', found: false, inSync: false };
  }

  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  const version = config.version;

  return {
    file: 'src-tauri/tauri.conf.json',
    found: !!version,
    version: version || 'NOT FOUND',
    inSync: version === expectedVersion
  };
}

function checkPackageLock(expectedVersion) {
  const lockPath = path.join(__dirname, '..', 'package-lock.json');
  
  if (!fs.existsSync(lockPath)) {
    return { file: 'package-lock.json', found: false, inSync: true }; // Optional file
  }

  const lockFile = JSON.parse(fs.readFileSync(lockPath, 'utf8'));
  const version = lockFile.version;

  return {
    file: 'package-lock.json',
    found: !!version,
    version: version || 'NOT FOUND',
    inSync: !version || version === expectedVersion // OK if missing
  };
}

if (require.main === module) {
  main();
}

module.exports = { main };
