# Linux 安装指南

本文档提供了在不同 Linux 发行版上安装 Wallet Manager 的详细说明和常见问题解决方案。

## 系统要求

- Linux 发行版：Ubuntu 18.04+、Debian 10+、CentOS 8+、Fedora 30+ 或其他兼容发行版
- 架构：x86_64 (amd64) 或 aarch64 (arm64)
- 内存：至少 512MB RAM
- 存储：至少 100MB 可用空间

## 依赖项说明

Wallet Manager 基于 Tauri 框架开发，需要以下系统依赖：

- **libwebkit2gtk-4.1-0**: WebKit2 GTK+ 库（用于渲染 Web 内容）
- **libgtk-3-0**: GTK+ 3 图形界面库
- **libayatana-appindicator3-1**: 系统托盘指示器库

## 安装方法

### 方法一：直接安装 DEB 包（推荐）

1. 下载最新的 `.deb` 安装包
2. 安装依赖项（见下方各发行版说明）
3. 安装软件包：
   ```bash
   sudo dpkg -i wallet-manager_*.deb
   ```

### 方法二：使用 AppImage（通用）

1. 下载 `.AppImage` 文件
2. 添加执行权限：
   ```bash
   chmod +x wallet-manager_*.AppImage
   ```
3. 直接运行：
   ```bash
   ./wallet-manager_*.AppImage
   ```

## 各发行版依赖安装

### Ubuntu / Debian

#### Ubuntu 24.04+ / Debian 12+
```bash
# 更新包列表
sudo apt update

# 安装依赖
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0 libayatana-appindicator3-1
```

#### Ubuntu 22.04 / Debian 11
```bash
# 更新包列表
sudo apt update

# 安装依赖（可能需要添加额外源）
sudo apt install libwebkit2gtk-4.0-37 libgtk-3-0 libayatana-appindicator3-1

# 如果 libwebkit2gtk-4.0-37 不可用，尝试：
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0 libayatana-appindicator3-1
```

#### Ubuntu 20.04 / Debian 10
```bash
# 更新包列表
sudo apt update

# 安装依赖
sudo apt install libwebkit2gtk-4.0-37 libgtk-3-0 libappindicator3-1
```

### CentOS / RHEL / Rocky Linux

#### CentOS 8+ / RHEL 8+ / Rocky Linux 8+
```bash
# 启用 EPEL 仓库
sudo dnf install epel-release

# 安装依赖
sudo dnf install webkit2gtk3 gtk3 libappindicator-gtk3
```

#### CentOS 7 / RHEL 7
```bash
# 启用 EPEL 仓库
sudo yum install epel-release

# 安装依赖
sudo yum install webkit2gtk3 gtk3 libappindicator-gtk3
```

### Fedora

```bash
# 安装依赖
sudo dnf install webkit2gtk4.1 gtk3 libappindicator-gtk3
```

### Arch Linux / Manjaro

```bash
# 安装依赖
sudo pacman -S webkit2gtk-4.1 gtk3 libappindicator-gtk3
```

### openSUSE

```bash
# 安装依赖
sudo zypper install webkit2gtk3 gtk3 libappindicator3-1
```

## 常见问题解决方案

### 问题 1: libwebkit2gtk-4.0-37 不可安装

**错误信息：**
```
wallet-manager : Depends: libwebkit2gtk-4.0-37 but it is not installable
```

**解决方案：**

1. **方案一：安装新版本依赖**
   ```bash
   sudo apt install libwebkit2gtk-4.1-0
   ```

2. **方案二：手动修复依赖**
   ```bash
   # 下载并解包 deb 文件
   dpkg-deb -x wallet-manager_*.deb wallet-manager-extracted
   dpkg-deb --control wallet-manager_*.deb wallet-manager-control
   
   # 编辑控制文件
   sed -i 's/libwebkit2gtk-4.0-37/libwebkit2gtk-4.1-0/g' wallet-manager-control/control
   
   # 重新打包
   dpkg-deb -b wallet-manager-extracted wallet-manager-fixed.deb
   cp -r wallet-manager-control/* wallet-manager-extracted/DEBIAN/
   dpkg-deb -b wallet-manager-extracted wallet-manager-fixed.deb
   
   # 安装修复后的包
   sudo dpkg -i wallet-manager-fixed.deb
   ```

3. **方案三：使用 AppImage**
   如果依赖问题无法解决，建议使用 AppImage 版本，它包含了所有必要的依赖。

### 问题 2: 缺少 libappindicator

**错误信息：**
```
Error: libappindicator3.so.1: cannot open shared object file
```

**解决方案：**
```bash
# Ubuntu/Debian
sudo apt install libayatana-appindicator3-1
# 或者
sudo apt install libappindicator3-1

# CentOS/RHEL/Fedora
sudo dnf install libappindicator-gtk3
```

### 问题 3: GTK 版本不兼容

**错误信息：**
```
Gtk-WARNING **: Theme parsing error
```

**解决方案：**
```bash
# 安装完整的 GTK3 主题
sudo apt install gtk3-engines-breeze  # 或其他主题包

# 设置 GTK 主题
export GTK_THEME=Adwaita:dark  # 或其他可用主题
```

### 问题 4: 权限问题

**错误信息：**
```
Permission denied
```

**解决方案：**
```bash
# 确保文件有执行权限
chmod +x wallet-manager

# 如果是 AppImage
chmod +x wallet-manager_*.AppImage
```

### 问题 5: 显示问题（Wayland）

如果在 Wayland 环境下遇到显示问题：

```bash
# 强制使用 X11
export GDK_BACKEND=x11
./wallet-manager

# 或者设置 Wayland 兼容
export WAYLAND_DISPLAY=wayland-0
./wallet-manager
```

## 验证安装

安装完成后，可以通过以下方式验证：

1. **命令行启动：**
   ```bash
   wallet-manager
   ```

2. **检查依赖：**
   ```bash
   ldd $(which wallet-manager) | grep -E "webkit|gtk|appindicator"
   ```

3. **查看版本信息：**
   ```bash
   wallet-manager --version
   ```

## 卸载

### DEB 包卸载
```bash
sudo apt remove wallet-manager
# 或者
sudo dpkg -r wallet-manager
```

### AppImage 卸载
直接删除 AppImage 文件即可：
```bash
rm wallet-manager_*.AppImage
```

## 技术支持

如果遇到其他问题，请：

1. 检查系统日志：`journalctl -f`
2. 查看应用日志：`~/.local/share/wallet-manager/logs/`
3. 提交 Issue：[GitHub Issues](https://github.com/ezban/wallet_manager/issues)

## 更新日志

- **v0.1.0**: 初始版本，修复了 libwebkit2gtk-4.0-37 依赖问题

---

**注意：** 本文档会随着软件更新而更新，建议定期查看最新版本。