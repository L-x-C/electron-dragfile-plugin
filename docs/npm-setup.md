# NPM 和 GitHub Actions 认证设置指南

## 🔧 设置 NPM Token

### 步骤 1：创建 NPM Access Token

1. 登录 [npmjs.com](https://www.npmjs.com)
2. 点击右上角头像 → **Access Tokens**
3. 点击 **Generate New Token**
4. 选择 **Classic Token** 类型（GitHub Actions 需要 Classic Token）
5. 给 token 一个描述性名称，如 "GitHub Actions electron-dragfile-plugin"
6. 复制生成的 token（只显示一次，请妥善保存）

### 步骤 2：在 GitHub 仓库中设置 Secrets

1. 进入你的 GitHub 仓库：https://github.com/L-x-C/electron-dragfile-plugin
2. 点击 **Settings** 标签页
3. 在左侧菜单中找到 **Secrets and variables** → **Actions**
4. 点击 **New repository secret**
5. 添加以下 secrets：

#### NPM_TOKEN
- **Name**: `NPM_TOKEN`
- **Secret**: [粘贴刚才创建的 NPM Access Token]

#### (可选) 如果需要自定义 GitHub Release
- **Name**: `GITHUB_TOKEN` (通常不需要，GitHub 会自动提供)

## 🚀 重新触发发布

设置完成后：

1. 删除并重新创建标签来触发新的发布：
```bash
git tag -d v1.0.5
git push origin :refs/tags/v1.0.5
git tag v1.0.5
git push origin v1.0.5
```

2. 或者创建一个新版本：
```bash
git tag v1.0.6
git push origin v1.0.6
```

## 🔍 验证设置

设置完成后，GitHub Actions 应该能够：
- ✅ 自动发布到 npm
- ✅ 创建 GitHub Release
- ✅ 上传二进制文件到 Release

## 📝 注意事项

- **安全性**: 不要在代码或提交中包含真实的 token
- **权限**: 确保 NPM token 有发布权限
- **有效期**: NPM Automation tokens 没有过期时间
- **范围**: Token 有权限发布所有属于你的包

## 🛠️ 故障排除

如果发布仍然失败：

1. **检查 NPM 权限**: 确保你有这个包的发布权限
2. **检查 token 类型**: 必须是 **Classic Token** 类型，不是 **Granular**
3. **检查仓库设置**: 确保 Actions 已启用
4. **查看日志**: 检查 GitHub Actions 的详细错误信息

---

📞 需要帮助？检查 GitHub Actions 日志页面获取详细错误信息。