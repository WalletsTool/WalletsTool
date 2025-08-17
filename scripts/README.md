# Scripts 目录

本目录包含项目的各种脚本工具。

## update-version.js

版本升级脚本，用于自动更新项目版本号并创建 git 标签。

### 功能

- 更新 `package.json` 中的版本号
- 更新 `src-tauri/Cargo.toml` 中的版本号
- 更新 `src-tauri/Cargo.lock` 中的版本号（如果存在）
- 自动提交修改的文件并推送到远程仓库
- 创建 git 标签（格式：`v{version}`）
- 推送标签到远程仓库

### 使用方法

```bash
# 方法1：直接运行脚本（支持v前缀）
node scripts/update-version.js v1.2.3
node scripts/update-version.js 1.2.3

# 方法2：使用 npm 脚本
npm run version:update v1.2.3
```

### 参数说明

- `version`: 新的版本号，支持v前缀（如：v1.2.3、1.2.3、v2.0.0-beta.1）

### 注意事项

1. 运行前建议先提交所有未提交的更改
2. 脚本会检查版本号格式的有效性
3. 如果标签已存在，脚本会报错并停止执行
4. 脚本会自动执行以下操作：
   - 添加修改的文件到暂存区
   - 创建提交（格式："chore: bump version to vx.x.x"）
   - 推送更改到远程仓库
   - 创建并推送版本标签

### 示例

```bash
# 升级到版本 0.3.0（支持v前缀）
node scripts/update-version.js v0.3.0
node scripts/update-version.js 0.3.0

# 升级到预发布版本
node scripts/update-version.js v1.0.0-beta.1

# 查看帮助信息
node scripts/update-version.js --help
```