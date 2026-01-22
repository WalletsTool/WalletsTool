import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

function execGitCommand(command) {
  try {
    return execSync(command, { cwd: projectRoot, encoding: 'utf8' }).trim();
  } catch (error) {
    return null;
  }
}

function getCommitLogs(fromTag) {
  const range = fromTag ? `${fromTag}..HEAD` : 'HEAD';
  const logs = execGitCommand(`git log ${range} --pretty=format:"%s"`);
  return logs ? logs.split('\n') : [];
}

function parseCommits(commits) {
  const categories = {
    feat: { title: '✨ New Features | 新功能', items: [] },
    fix: { title: '🐛 Bug Fixes | 修复', items: [] },
    perf: { title: '⚡ Performance | 性能优化', items: [] },
    refactor: { title: '♻️ Refactoring | 重构', items: [] },
    docs: { title: '📝 Documentation | 文档', items: [] },
    chore: { title: '🔧 Chore | 杂项', items: [] },
    other: { title: 'Other | 其他', items: [] }
  };

  const regex = /^(\w+)(?:\((.+)\))?:\s*(.+)$/;

  commits.forEach(commit => {
    // 过滤掉版本升级的自动提交
    if (commit.includes('bump version to v')) {
      return;
    }

    const match = commit.match(regex);
    if (match) {
      const type = match[1].toLowerCase();
      const scope = match[2];
      const message = match[3];
      const formattedMessage = scope ? `**${scope}:** ${message}` : message;

      if (categories[type]) {
        categories[type].items.push(formattedMessage);
      } else {
        categories.other.items.push(commit);
      }
    } else {
      categories.other.items.push(commit);
    }
  });

  return categories;
}

function generateMarkdown(categories) {
  let markdown = '';

  for (const key in categories) {
    const category = categories[key];
    if (category.items.length > 0) {
      markdown += `### ${category.title}\n\n`;
      category.items.forEach(item => {
        markdown += `- ${item}\n`;
      });
      markdown += '\n';
    }
  }

  return markdown;
}

function main() {
  try {
    // 获取最近的 tag
    let lastTag = execGitCommand('git describe --tags --abbrev=0');
    console.log(`ℹ️  当前检测到的最近标签: ${lastTag || 'None'}`);
    
    let fromRef = lastTag;
    let toRef = 'HEAD';

    // 检查是否有新的提交
    const diff = execGitCommand(`git rev-list ${lastTag}..HEAD --count`);
    
    if (diff === '0') {
        console.log('ℹ️  当前 HEAD 与最近标签一致，尝试获取上一版本变更...');
        // 获取上一个 tag
        try {
              // git describe --tags --abbrev=0 HEAD^ 可能会失败如果只有一个 tag
              // Windows cmd 中 ^ 是转义符，使用 ~1 代替
              const prevTag = execGitCommand(`git describe --tags --abbrev=0 ${lastTag}~1`);
              if (prevTag) {
                  fromRef = prevTag;
                 toRef = lastTag; // 或者 HEAD，因为它们一样
                 console.log(`ℹ️  将生成从 ${fromRef} 到 ${toRef} 的变更日志`);
             } else {
                 console.log('⚠️  未找到更早的标签，无法生成演示日志');
             }
        } catch (e) {
            console.log('⚠️  获取上一标签失败');
        }
    } else {
        console.log(`ℹ️  将生成从 ${fromRef} 到 ${toRef} 的变更日志`);
    }

    const logs = execGitCommand(`git log ${fromRef}..${toRef} --pretty=format:"%s"`);
    const commits = logs ? logs.split('\n') : [];
    
    console.log(`ℹ️  发现 ${commits.length} 个提交`);

    const categories = parseCommits(commits);
    const markdown = generateMarkdown(categories);

    console.log('\n=== 生成的 Release Notes 预览 ===\n');
    console.log(markdown);
    console.log('===================================\n');

    // 写入 RELEASE_NOTES.md
    const outputPath = path.join(projectRoot, 'RELEASE_NOTES.md');
    fs.writeFileSync(outputPath, markdown);
    console.log(`✅ 已生成 RELEASE_NOTES.md`);

  } catch (error) {
    console.error('❌ 生成失败:', error);
  }
}

main();
