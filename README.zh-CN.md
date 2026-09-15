# Jot

<p align="center">
  <img src="./resources/jot.svg" width="72" height="72" alt="Jot logo">
</p>

<p align="center">轻量、本地优先的桌面笔记应用，用于快速捕捉想法、任务和灵感。</p>

<p align="center">
  <a href="./README.md">English</a> ·
  <a href="https://github.com/yumkea/jot-rust/releases">发行版本</a> ·
  <a href="https://github.com/yumkea/jot-rust/issues">问题反馈</a> ·
  <a href="./LICENSE">GPL-3.0</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.x-24c8db?style=flat-square" alt="Tauri 2">
  <img src="https://img.shields.io/badge/Vue-3-42b883?style=flat-square" alt="Vue 3">
  <img src="https://img.shields.io/badge/TypeScript-5-3178c6?style=flat-square" alt="TypeScript 5">
  <img src="https://img.shields.io/badge/License-GPL--3.0-4f46e5?style=flat-square" alt="GPL-3.0">
</p>

## Jot 是什么？

Jot 是一个专注于快速记录的浮动笔记窗口。它以 Vue 构建编辑体验、以 Tauri/Rust 提供桌面能力；笔记内容保存在本地 Markdown 文件中，SQLite 仅用于保存应用偏好设置。

- 通过全局快捷键和轻量窗口快速记录。
- 使用 Tiptap 编写富文本、Markdown 友好的笔记。
- 每篇笔记都保存为带 Frontmatter 元数据的 `.md` 文件，便于迁移和备份。
- 提供托盘、窗口操作、文件选择和可配置快捷键等原生桌面功能。

<p align="center">
  <img src="./assets/main_window.png" width="42%" alt="Jot 浅色界面">
  <img src="./assets/main_window_white.png" width="42%" alt="Jot 深色界面">
</p>

## 功能

- 笔记创建、搜索、历史记录、大纲导航和笔记设置。
- 基于 Tiptap 的富文本编辑与 Markdown 转换。
- 表格、任务列表、KaTeX 公式、代码块和常用格式快捷键。
- 使用 `id`、`title`、`created_at`、`updated_at` Frontmatter 持久化笔记。
- 支持自定义笔记目录，并可从旧版 SQLite 笔记库导入。
- 集成 Tauri 原生托盘、窗口、对话框和全局快捷键能力。

<p align="center">
  <img src="./assets/task_table_latex.png" width="42%" alt="任务表格和 LaTeX 编辑">
  <img src="./assets/outline.png" width="42%" alt="笔记大纲导航">
</p>

## 架构与设计文档

以下图表经过代码证据核验与桌面浏览器检查。PNG 预览可直接在 GitHub 查看；打开链接的 HTML 文件可在本地使用主题切换、缩放、搜索和关系追踪。

### 系统架构

[打开交互式系统架构图](./docs/architecture/jot-rust-architecture.html)

<a href="./docs/architecture/jot-rust-architecture.html">
  <img src="./docs/architecture/jot-rust-architecture.visual-check.1440x900.light.png" alt="Jot 系统架构图">
</a>

### 笔记数据流

[打开交互式数据流图](./docs/architecture/jot-rust-dataflow.html)

<a href="./docs/architecture/jot-rust-dataflow.html">
  <img src="./docs/architecture/jot-rust-dataflow.visual-check.1440x900.light.png" alt="Jot 笔记数据流图">
</a>

### 笔记生命周期

[打开交互式生命周期图](./docs/architecture/jot-rust-note-lifecycle.html)

<a href="./docs/architecture/jot-rust-note-lifecycle.html">
  <img src="./docs/architecture/jot-rust-note-lifecycle.visual-check.1440x900.light.png" alt="Jot 笔记生命周期图">
</a>

图表规格与 HTML 成品位于 [`docs/architecture`](./docs/architecture)。

## 笔记存储格式

笔记会写入用户可配置目录中的 Markdown 文件：

```markdown
---
id: "..."
title: "..."
created_at: 2026-07-02 10:00:00
updated_at: 2026-07-02 10:30:00
---

你的笔记内容...
```

应用设置（如笔记目录和快捷键）保存在独立的本地 SQLite 数据库中。已有 SQLite 笔记数据库可通过应用导入到 Markdown 目录。

## 开始使用

### 环境要求

- Node.js 20.19+
- npm
- Rust stable
- 平台相关的 [Tauri 前置依赖](https://v2.tauri.app/start/prerequisites/)

### 本地开发

```bash
git clone git@github.com:yumkea/jot-rust.git
cd jot-rust
npm install
npm run tauri dev
```

仅运行 Vite 前端：

```bash
npm run dev
```

开发服务器地址为 `http://localhost:1420`。

## 常用命令

```bash
npm run dev          # 启动 Vite
npm run build        # 类型检查并构建前端
npm test             # 运行 Vitest
npm run tauri dev    # 启动桌面端开发环境
npm run tauri build  # 构建桌面安装包
```

## 项目结构

```text
src/                 Vue 界面、编辑器组件、组合式逻辑和 Tauri API 封装
src-tauri/           Rust 命令、原生集成、存储和 Tauri 配置
resources/           应用图标与资源生成脚本
docs/architecture/   架构、数据流和生命周期图
```

## 贡献

欢迎提交 Issue 和 Pull Request。提交前请运行：

```bash
npm test
npm run build
```

## 许可证

本项目使用 [GNU General Public License v3.0](./LICENSE) 许可证。
