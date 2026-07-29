# AuraVoice

一个基于 Microsoft Edge TTS 服务的桌面文本转语音（TTS）应用程序，扩展了英语口语练习功能。采用 Vue 3、Vuetify 和 Tauri 构建。

## 功能特性

### 1. 文本转语音（TTS）
- 支持实时文本输入和语音转换
- 可调节语速、音调和音量
- 多种音色选择（基于 Edge TTS 支持的语音）
- 内置音频播放器，支持播放、暂停、停止
- 生成后可保存音频文件（支持 MP3、WAV、OGG、FLAC 格式）
- **一键发送到跟读练习** — 将输入文本直接导入影子跟读模式

### 2. 批量转换
- 支持批量上传文本文件（.txt、.md、.markdown、.docx）
- 拖拽上传文件
- 可配置并发转换数量
- 实时查看转换进度
- 支持单文件重试
- 自定义输出格式和保存路径

### 3. 🎯 英语口语练习（新增）
利用 Whisper 本地语音识别实现完整的听说反馈闭环。

#### 影子跟读（Shadowing）
- 播放 Edge TTS 原音 → 用户跟读录音 → Whisper 语音识别 → WER 评分反馈
- 逐词颜色标记（正确/错误/遗漏/多余）
- 内置场景库（咖啡馆点单、机场值机、工作面试等）
- 支持导入自定义文本或文档进行跟读

#### 场景对话（Role-play）
- 模拟真实对话场景（酒店入住、餐厅点餐、看医生等）
- TTS 播放对方台词，用户录音回答
- 逐轮评分反馈

#### 练习历史与统计
- SQLite 持久化存储所有练习记录
- 历史会话列表与详情回放
- 得分趋势、总练习时长等统计

### 4. 个性化设置
- **输出设置**
  - 自定义保存路径
  - 默认输出格式（MP3、WAV、OGG、FLAC）
  - 显示语言切换（简体中文/English）
  - 转换后自动播放开关

- **处理设置**
  - 最大重试次数（1-10 次）
  - 文件并发数（1-5）
  - 分段并发数（1-5）

- **Whisper 语音识别配置**
  - whisper-cli 二进制路径
  - Whisper 模型文件路径（支持 base.en / small.en / tiny.en）
  - 一键下载模型按钮

### 5. 界面特性
- 现代化的玻璃拟态（Glassmorphism）设计
- 支持深色/浅色主题切换
- 自定义窗口标题栏
- 响应式布局
- 国际化支持（i18n）

## 技术栈

### 前端
- **Vue 3** — 渐进式 JavaScript 框架
- **TypeScript** — 类型安全的 JavaScript
- **Vuetify 3** — Material Design 组件库
- **Vue Router** — 官方路由管理器
- **Pinia** — Vue 状态管理库
- **Vue I18n** — 国际化插件

### 后端（Tauri）
- **Rust** — 系统级编程语言
- **Tauri 2** — 跨平台桌面应用框架
- **Tokio** — 异步运行时
- **reqwest** — HTTP 客户端
- **FFmpeg** — 音频格式转换

### 口语练习核心
- **Whisper.cpp** — 本地语音识别（CLI sidecar 模式）
- **cpal** — 跨平台音频采集
- **hound** — WAV 编码
- **WER 评分算法** — Levenshtein 编辑距离逐词对比
- **SQLite** — 练习数据持久化（`rusqlite`）

### 其他
- **Vite** — 前端构建工具
- **Vitest** — 单元测试框架
- **Happy DOM** — 轻量级 DOM 实现

## 运行项目

### 环境要求

- **Node.js** >= 18.0.0
- **pnpm** >= 8.0.0
- **Rust** >= 1.70.0
- **系统依赖**
  - Windows: 无需额外依赖
  - Linux: `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `file`, `x11-utils`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
  - macOS: Xcode Command Line Tools

### 安装依赖

```bash
# 克隆项目
git clone <repository-url>
cd auravioce

# 安装前端依赖
pnpm install

# 同步 FFmpeg 二进制文件（自动执行）
pnpm ffmpeg:sync
```

### 开发模式

```bash
# 启动开发服务器
pnpm tauri dev
```

这将会：
1. 启动 Vite 开发服务器（http://localhost:1420）
2. 编译并运行 Tauri 应用窗口

### 生产构建

```bash
# 构建前端
pnpm build

# 构建桌面应用
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

### 运行测试

```bash
# 运行所有 Rust 测试
cd src-tauri && cargo test

# 运行前端测试
pnpm test

# 运行测试并生成覆盖率报告
pnpm test:coverage
```

## Whisper 配置

口语练习的评分功能需要 Whisper 本地模型。有两种方式：

### 方式一：通过设置页面（推荐）
1. 下载 [whisper-cli](https://github.com/ggerganov/whisper.cpp/releases) 和模型文件
2. 打开应用 → Settings → Whisper Speech Recognition
3. 设置 whisper-cli 路径和模型文件路径

### 方式二：环境变量
```bash
set TTS_VUE_NEXT_WHISPER_PATH=C:/tools/whisper.cpp/whisper-cli.exe
set TTS_VUE_NEXT_WHISPER_MODEL=C:/models/ggml-base.en.bin
```

推荐使用 `ggml-base.en.bin`（~142MB），在 i7-13700K 上每句推理约 1-3 秒。

## 项目结构

```
auravioce/
├── src/                            # 前端源代码
│   ├── components/
│   │   ├── batch/                 # 批量转换组件
│   │   ├── layout/                # 布局组件
│   │   ├── practice/              # 口语练习组件
│   │   │   ├── shadowing/         # 影子跟读
│   │   │   ├── roleplay/          # 场景对话
│   │   │   └── shared/            # 共享组件（录音、评分、波形等）
│   │   └── tts/                   # TTS 组件
│   ├── locales/                   # 国际化文件
│   ├── plugins/                   # Vue 插件
│   ├── router/                    # 路由配置
│   ├── stores/                    # Pinia 状态管理
│   │   └── practice/              # 口语练习 Store
│   ├── types/                     # TypeScript 类型定义
│   ├── utils/                     # 工具函数
│   ├── views/                     # 页面视图
│   │   └── practice/              # 口语练习页面
│   ├── App.vue                    # 根组件
│   └── main.ts                    # 入口文件
├── src-tauri/                     # Tauri 后端源代码
│   ├── crates/                    # 工作空间 Crate
│   │   ├── practice-domain/       # 领域模型（纯类型）
│   │   ├── audio-engine/          # 音频录制引擎（cpal）
│   │   ├── speech-engine/         # 语音识别接口（Provider Trait）
│   │   ├── assessment-engine/     # 发音评估引擎（WER）
│   │   ├── content-engine/        # 场景内容管理
│   │   └── storage/               # SQLite 持久化
│   ├── src/
│   │   ├── commands/              # Tauri 命令
│   │   │   ├── practice.rs        # 口语练习命令
│   │   │   ├── tts.rs             # TTS 转换命令
│   │   │   ├── voices.rs          # 语音列表命令
│   │   │   ├── audio.rs           # 音频处理命令
│   │   │   └── file.rs            # 文件操作命令
│   │   ├── speech/                # Whisper 语音识别
│   │   │   └── whisper.rs         # CLI + Dummy 识别器
│   │   ├── edge_tts/              # Edge TTS 核心逻辑
│   │   ├── audio/                 # 音频处理（FFmpeg）
│   │   └── utils/                 # 工具函数
│   ├── assets/
│   │   └── scenarios/             # 预置场景 JSON 文件
│   ├── binaries/                  # 二进制文件（FFmpeg，Whisper）
│   ├── Cargo.toml                 # Rust 依赖配置
│   └── tauri.conf.json            # Tauri 配置
├── docs/                          # 项目文档
│   └── superpowers/specs/         # 设计文档和测试计划
├── package.json                   # Node.js 依赖配置
├── tsconfig.json                  # TypeScript 配置
└── vite.config.ts                 # Vite 配置
```

## 路由

| 路径 | 页面 | 说明 |
|------|------|------|
| `/` | 文本转语音 | 单文本 TTS 转换 |
| `/batch` | 批量转换 | 批量文件转换 |
| `/practice` | 练习中心 | 口语练习模式选择 |
| `/practice/shadowing` | 影子跟读 | 逐句跟读训练 |
| `/practice/roleplay` | 场景对话 | 对话模拟练习 |
| `/practice/history` | 练习历史 | 历史记录与统计 |
| `/practice/history/:id` | 会话详情 | 单次练习详情回放 |
| `/settings` | 设置 | 应用偏好与 Whisper 配置 |

## 核心功能说明

### Edge TTS 集成

项目通过 WebSocket 连接 Microsoft Edge TTS 服务，实现了完整的文本转语音功能：

1. **语音获取** — 获取 Edge TTS 支持的所有可用语音
2. **DRM 认证** — 处理服务器的 DRM 挑战响应
3. **SSML 构建** — 构建标准化的语音合成标记语言
4. **文本分段** — 自动将长文本分割为适合处理的片段
5. **重试机制** — 内置重试逻辑，提高转换成功率

### 语音识别与评分

- **Provider Trait 设计** — `SpeechRecognizer` trait 支持多种实现（CLI、Dummy、云端）
- **Whisper CLI sidecar** — 调用 whisper.cpp 二进制进行本地推理
- **WER 评分** — 基于 Levenshtein 编辑距离的逐词对比
- **三层评分维度** — 准确度（Accuracy）+ 完整度（Completeness）+ 综合评分

### 音频处理

- 使用 FFmpeg 进行音频格式转换
- 支持多种输出格式（MP3、WAV、OGG、FLAC）
- 临时文件自动清理

### 状态管理

- **TTS Store** — 管理单个文本转换状态
- **Batch Store** — 管理批量转换队列和进度
- **Voices Store** — 管理可用语音列表
- **Settings Store** — 管理用户设置（持久化，含 Whisper 配置）
- **Practice Stores** — `session.ts`, `recorder.ts`, `scoring.ts`, `content.ts`

## 开发建议

### 推荐的 IDE

- **VS Code** + 以下扩展：
  - [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
  - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### 代码规范

项目遵循以下代码规范：
- 使用 TypeScript 进行类型检查
- 组件采用 Composition API 和 `<script setup>` 语法
- 状态管理使用 Pinia
- 样式使用 Vuetify 和 scoped CSS

## 测试

```bash
# Rust 后端测试（workspace crates）
cd src-tauri && cargo test -p practice-domain -p audio-engine -p speech-engine \
  -p assessment-engine -p content-engine -p storage

# 完整 Rust 测试
cd src-tauri && cargo test

# 前端测试
pnpm test

# TypeScript 类型检查
cd src-tauri/.. && npx vue-tsc --noEmit
```

## 许可证

本项目采用 MIT 许可证。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 🎉致谢
- 本项目在 [LINUX DO](https://linux.do/) 社区推广，感谢 LINUX DO 社区对开源项目的支持与认可。
- Edge TTS 服务由 Microsoft 提供
- Whisper 模型由 OpenAI 开源，whisper.cpp 由 [ggerganov](https://github.com/ggerganov/whisper.cpp) 维护
