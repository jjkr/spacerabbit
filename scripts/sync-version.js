#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

/**
 * Synchronizes version from package.json to all other version files
 * This ensures package.json remains the single source of truth for versioning
 */

function main() {
  try {
    // Read version from package.json
    const packageJsonPath = path.join(__dirname, '..', 'package.json');
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
    const version = packageJson.version;

    if (!version) {
      console.error('❌ No version found in package.json');
      process.exit(1);
    }

    console.log(`🔄 Syncing version ${version} to all files...`);

    // Update Cargo.toml
    updateCargoToml(version);
    
    // Update tauri.conf.json
    updateTauriConfig(version);
    
    // Update package-lock.json (if it exists)
    updatePackageLock(version);

    console.log('✅ Version synchronization complete!');
    console.log(`📦 All files now use version: ${version}`);

  } catch (error) {
    console.error('❌ Error syncing versions:', error.message);
    process.exit(1);
  }
}

function updateCargoToml(version) {
  const cargoPath = path.join(__dirname, '..', 'src-tauri', 'Cargo.toml');
  
  if (!fs.existsSync(cargoPath)) {
    console.warn('⚠️  Cargo.toml not found, skipping...');
    return;
  }

  let cargoContent = fs.readFileSync(cargoPath, 'utf8');
  
  // Replace version in [package] section
  cargoContent = cargoContent.replace(
    /^version\s*=\s*"[^"]*"/m,
    `version = "${version}"`
  );

  fs.writeFileSync(cargoPath, cargoContent);
  console.log('✓ Updated src-tauri/Cargo.toml');
}

function updateTauriConfig(version) {
  const configPath = path.join(__dirname, '..', 'src-tauri', 'tauri.conf.json');
  
  if (!fs.existsSync(configPath)) {
    console.warn('⚠️  tauri.conf.json not found, skipping...');
    return;
  }

  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  config.version = version;

  fs.writeFileSync(configPath, JSON.stringify(config, null, 2) + '\n');
  console.log('✓ Updated src-tauri/tauri.conf.json');
}

function updatePackageLock(version) {
  const lockPath = path.join(__dirname, '..', 'package-lock.json');
  
  if (!fs.existsSync(lockPath)) {
    console.log('ℹ️  package-lock.json not found, skipping...');
    return;
  }

  const lockFile = JSON.parse(fs.readFileSync(lockPath, 'utf8'));
  
  // Update root version
  if (lockFile.version) {
    lockFile.version = version;
  }
  
  // Update packages."" version (npm v7+ format)
  if (lockFile.packages && lockFile.packages[""]) {
    lockFile.packages[""].version = version;
  }

  fs.writeFileSync(lockPath, JSON.stringify(lockFile, null, 2) + '\n');
  console.log('✓ Updated package-lock.json');
}

if (require.main === module) {
  main();
}

module.exports = { main };
