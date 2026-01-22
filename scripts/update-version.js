#!/usr/bin/env node

import fs from 'fs/promises';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

/**
 * 验证版本号格式 (semantic versioning)
 * @param {string} version - 版本号（支持v前缀，如v1.2.3或1.2.3）
 * @returns {boolean} - 是否有效
 */
function isValidVersion(version) {
  const semverRegex = /^v?\d+\.\d+\.\d+(?:-[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*)?(?:\+[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*)?$/;
  return semverRegex.test(version);
}

/**
 * 提取纯版本号（去除v前缀）
 * @param {string} version - 版本号（可能带v前缀）
 * @returns {string} - 纯版本号
 */
function extractPureVersion(version) {
  return version.startsWith('v') ? version.slice(1) : version;
}

/**
 * 更新 package.json 文件中的版本号
 * @param {string} newVersion - 新版本号
 */
async function updatePackageJson(newVersion) {
  const packageJsonPath = path.join(projectRoot, 'package.json');
  
  try {
    const content = await fs.readFile(packageJsonPath, 'utf8');
    const packageJson = JSON.parse(content);
    
    const oldVersion = packageJson.version;
    packageJson.version = newVersion;
    
    await fs.writeFile(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');
    console.log(`✅ 已更新 package.json: ${oldVersion} → ${newVersion}`);
  } catch (error) {
    throw new Error(`更新 package.json 失败: ${error.message}`);
  }
}

/**
 * 更新 Cargo.toml 文件中的版本号
 * @param {string} newVersion - 新版本号
 */
async function updateCargoToml(newVersion) {
  const cargoTomlPath = path.join(projectRoot, 'src-tauri', 'Cargo.toml');
  
  try {
    const content = await fs.readFile(cargoTomlPath, 'utf8');
    
    // 匹配 [package] 部分的 version 字段
    const versionRegex = /(\[package\][\s\S]*?version\s*=\s*")[^"]*(")/ ;
    const oldVersionMatch = content.match(/version\s*=\s*"([^"]*)"/); 
    const oldVersion = oldVersionMatch ? oldVersionMatch[1] : 'unknown';
    
    const updatedContent = content.replace(versionRegex, `$1${newVersion}$2`);
    
    await fs.writeFile(cargoTomlPath, updatedContent);
    console.log(`✅ 已更新 Cargo.toml: ${oldVersion} → ${newVersion}`);
  } catch (error) {
    throw new Error(`更新 Cargo.toml 失败: ${error.message}`);
  }
}

/**
 * 更新 tauri.conf.json 文件中的版本号
 * @param {string} newVersion - 新版本号
 */
async function updateTauriConfig(newVersion) {
  const tauriConfigPath = path.join(projectRoot, 'src-tauri', 'tauri.conf.json');
  
  try {
    const content = await fs.readFile(tauriConfigPath, 'utf8');
    const tauriConfig = JSON.parse(content);
    
    const oldVersion = tauriConfig.version;
    tauriConfig.version = newVersion;
    
    await fs.writeFile(tauriConfigPath, JSON.stringify(tauriConfig, null, 2) + '\n');
    console.log(`✅ 已更新 tauri.conf.json: ${oldVersion} → ${newVersion}`);
  } catch (error) {
    throw new Error(`更新 tauri.conf.json 失败: ${error.message}`);
  }
}

/**
 * 更新 Cargo.lock 文件中的版本号
 * @param {string} newVersion - 新版本号
 */
async function updateCargoLock(newVersion) {
  const cargoLockPath = path.join(projectRoot, 'src-tauri', 'Cargo.lock');
  
  try {
    // 检查 Cargo.lock 是否存在
    await fs.access(cargoLockPath);
    
    const content = await fs.readFile(cargoLockPath, 'utf8');
    
    // 匹配项目名称对应的版本号（假设项目名为 WalletsTool）
    const projectNameRegex = /name\s*=\s*"WalletsTool"[\s\S]*?version\s*=\s*"[^"]*"/;
    const versionRegex = /(name\s*=\s*"WalletsTool"[\s\S]*?version\s*=\s*")[^"]*(")/ ;
    
    const oldVersionMatch = content.match(/name\s*=\s*"WalletsTool"[\s\S]*?version\s*=\s*"([^"]*)"/); 
    const oldVersion = oldVersionMatch ? oldVersionMatch[1] : 'unknown';
    
    if (projectNameRegex.test(content)) {
      const updatedContent = content.replace(versionRegex, `$1${newVersion}$2`);
      await fs.writeFile(cargoLockPath, updatedContent);
      console.log(`✅ 已更新 Cargo.lock: ${oldVersion} → ${newVersion}`);
    } else {
      console.log(`⚠️  Cargo.lock 中未找到项目版本信息，跳过更新`);
    }
  } catch (error) {
    if (error.code === 'ENOENT') {
      console.log(`⚠️  Cargo.lock 文件不存在，跳过更新`);
    } else {
      throw new Error(`更新 Cargo.lock 失败: ${error.message}`);
    }
  }
}

/**
 * 更新 release.yml 文件中的发布日志
 * @param {string} newVersion - 新版本号
 */
async function updateReleaseWorkflow(newVersion) {
  const releaseWorkflowPath = path.join(projectRoot, '.github', 'workflows', 'release.yml');
  const releaseNotesPath = path.join(projectRoot, 'RELEASE_NOTES.md');
  
  try {
    let notesContent = '请查看下方的变更日志了解本版本的新功能和修复。';
    
    // 检查是否存在 RELEASE_NOTES.md
    try {
      await fs.access(releaseNotesPath);
      const fileContent = await fs.readFile(releaseNotesPath, 'utf8');
      if (fileContent.trim()) {
        notesContent = fileContent.trim();
        console.log('✅ 已读取 RELEASE_NOTES.md 内容');
      }
    } catch (e) {
      console.log('ℹ️  未找到 RELEASE_NOTES.md，使用默认发布说明');
    }

    const workflowContent = await fs.readFile(releaseWorkflowPath, 'utf8');
    
    // 构造新的 releaseBody
    // 注意：YAML 多行字符串需要正确的缩进（这里是 12 个空格）
    const indent = '            ';
    const indentedNotes = notesContent.split('\n').join(`\n${indent}`);
    
    const newBody = `${indent}## 🎉 新版本发布 v${newVersion}
${indent}
${indent}### 📦 安装包下载
${indent}- **Windows**: 下载 \`.msi\` 或 \`.exe\` 文件
${indent}
${indent}### 🔄 更新说明
${indent}${indentedNotes}
${indent}
${indent}---
${indent}
${indent}See the assets below to download and install this version.
`;

    // 使用正则替换 releaseBody 内容
    // 匹配 releaseBody: | 到 releaseDraft: 之间的内容
    const regex = /(releaseBody: \|\n)([\s\S]*?)(          releaseDraft:)/;
    
    if (regex.test(workflowContent)) {
      const updatedContent = workflowContent.replace(regex, `$1${newBody}$3`);
      await fs.writeFile(releaseWorkflowPath, updatedContent);
      console.log(`✅ 已更新 release.yml 发布日志`);
    } else {
      console.warn('⚠️  无法在 release.yml 中找到 releaseBody 字段，跳过更新');
    }
    
  } catch (error) {
    throw new Error(`更新 release.yml 失败: ${error.message}`);
  }
}

/**
 * 执行 git 命令
 * @param {string} command - git 命令
 * @returns {string} - 命令输出
 */
function execGitCommand(command) {
  try {
    return execSync(command, { 
      cwd: projectRoot, 
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe']
    }).trim();
  } catch (error) {
    throw new Error(`Git 命令执行失败: ${command}\n${error.message}`);
  }
}

/**
 * 检查是否有未提交的更改
 */
function checkGitStatus() {
  try {
    const status = execGitCommand('git status --porcelain');
    if (status) {
      console.log('⚠️  检测到未提交的更改:');
      console.log(status);
      console.log('建议先提交所有更改后再创建版本标签。');
    }
  } catch (error) {
    console.log('⚠️  无法检查 git 状态，请确保在 git 仓库中运行此脚本');
  }
}

/**
 * 提交修改的文件并推送到远端
 * @param {string} version - 版本号
 */
function commitAndPushChanges(version) {
  try {
    // 添加修改的文件
    console.log('📝 添加修改的文件到暂存区...');
    execGitCommand('git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/Cargo.lock .github/workflows/release.yml RELEASE_NOTES.md');
    console.log('✅ 已添加修改的文件到暂存区');
    
    // 创建提交
    const commitMessage = `chore: bump version to v${version}`;
    console.log(`📝 创建提交: ${commitMessage}`);
    execGitCommand(`git commit -m "${commitMessage}"`);
    console.log('✅ 已创建提交');
    
    // 推送到远端
    console.log('📝 推送更改到远端...');
    execGitCommand('git push');
    console.log('✅ 已推送更改到远端');
    
  } catch (error) {
    throw new Error(`Git 提交和推送失败: ${error.message}`);
  }
}

/**
 * 创建并推送 git 标签
 * @param {string} version - 版本号
 */
function createAndPushTag(version) {
  const tagName = `v${version}`;
  
  try {
    // 检查标签是否已存在
    try {
      execGitCommand(`git rev-parse ${tagName}`);
      throw new Error(`标签 ${tagName} 已存在`);
    } catch (error) {
      if (!error.message.includes('已存在')) {
        // 标签不存在，继续创建
      } else {
        throw error;
      }
    }
    
    // 创建标签
    console.log(`📝 创建标签: ${tagName}`);
    execGitCommand(`git tag -a ${tagName} -m "Release version ${version}"`);
    console.log(`✅ 已创建标签: ${tagName}`);
    
    // 推送标签到远端
    console.log(`📝 推送标签到远端: ${tagName}`);
    execGitCommand(`git push origin ${tagName}`);
    console.log(`✅ 已推送标签到远端: ${tagName}`);
    
  } catch (error) {
    throw new Error(`Git 标签操作失败: ${error.message}`);
  }
}

/**
 * 主函数
 */
async function main() {
  try {
    // 获取命令行参数
    const args = process.argv.slice(2);
    
    if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
      console.log('📦 版本升级脚本');
      console.log('\n用法: node scripts/update-version.js <version>');
      console.log('\n参数:');
      console.log('  <version>    新版本号（支持v前缀，如v1.2.3或1.2.3）');
      console.log('\n示例:');
      console.log('  node scripts/update-version.js v1.2.3');
      console.log('  node scripts/update-version.js 1.2.3');
      console.log('\n功能:');
      console.log('  - 更新 package.json 中的版本号');
      console.log('  - 更新 Cargo.toml 中的版本号');
      console.log('  - 更新 tauri.conf.json 中的版本号');
      console.log('  - 更新 Cargo.lock 中的版本号');
      console.log('  - 提交修改的文件并推送到远端');
      console.log('  - 创建并推送 Git 标签');
      process.exit(0);
    }
    
    const inputVersion = args[0];
    
    // 验证版本号格式
    if (!isValidVersion(inputVersion)) {
      console.error('❌ 错误: 版本号格式无效，请使用语义化版本格式 (例如: v1.2.3 或 1.2.3)');
      console.log('\n使用 --help 查看详细用法');
      process.exit(1);
    }
    
    // 提取纯版本号用于文件更新
    const newVersion = extractPureVersion(inputVersion);
    
    console.log(`🚀 开始更新项目版本到: ${inputVersion}`);
    console.log('=' .repeat(50));
    
    // 检查是否有未提交的更改
    checkGitStatus();

    // 自动生成 RELEASE_NOTES.md
    console.log('\n📝 正在根据 Git 提交记录生成 RELEASE_NOTES.md...');
    try {
      execSync('node scripts/generate-release-notes.js', { 
        cwd: projectRoot, 
        stdio: 'inherit' 
      });
    } catch (error) {
      console.warn('⚠️  自动生成 RELEASE_NOTES.md 失败，将使用现有文件或默认内容');
    }
    
    // 更新各个文件中的版本号
    await updatePackageJson(newVersion);
    await updateCargoToml(newVersion);
    await updateTauriConfig(newVersion);
    await updateCargoLock(newVersion);
    
    // 更新 release.yml 中的发布日志
    await updateReleaseWorkflow(newVersion);
    
    console.log('\n📝 版本号更新完成，准备提交更改...');
    
    // 提交修改的文件并推送到远端
    await commitAndPushChanges(newVersion);
    
    console.log('\n📝 准备创建 git 标签...');
    
    // 创建并推送 git 标签
    createAndPushTag(newVersion);
    
    console.log('\n🎉 版本升级完成!');
    console.log(`新版本: v${newVersion}`);
    console.log('\n✅ 已完成的操作:');
    console.log('1. ✅ 更新了版本号文件');
    console.log('2. ✅ 提交并推送了更改');
    console.log('3. ✅ 创建并推送了版本标签');
    
  } catch (error) {
    console.error(`\n❌ 版本升级失败: ${error.message}`);
    process.exit(1);
  }
}

// 运行主函数
main();