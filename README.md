# LLM Chat Export

A terminal (TUI) tool for exporting LLM conversation history to local Markdown files.

Supports multiple platforms, keyword & date filtering, bilingual interface (Chinese / English).

## Supported Platforms

| Platform | Auth Method | Data Source |
|----------|------------|-------------|
| ChatGPT | Access Token | OpenAI API |
| Claude Web | Session Key | Claude.ai API |
| Claude Code | None (local) | `~/.claude/projects/` JSONL files |
| Cursor | None (local) | `~/Library/Application Support/Cursor/` SQLite DB |

## Features

- **Multi-platform** — ChatGPT, Claude Web, Claude Code, Cursor
- **Keyword filter** — Include / Exclude mode, comma-separated, matches conversation title
- **Date range filter** — Start date and end date (YYYY-MM-DD), either optional
- **Batch export** — Space to toggle, `a` to select all, Enter to download
- **Bilingual UI** — Chinese and English, selectable at startup
- **Step-by-step navigation** — Esc goes back one step, not jumping to the beginning
- **Re-export without restart** — After export completes, press Enter to go back and export more
- **Cross-platform binary** — macOS and Windows

## Usage

Run in terminal:

```bash
./llm-chat-export
```

### Flow

```
Language Select → Platform Select → [Token Input] → Filter → Loading → Conversation List → Downloading → Done
                                                                              ↑ f: re-filter     ↑ Enter: back
```

### Key Bindings

| Key | Action |
|-----|--------|
| Up / Down / j / k | Navigate |
| Enter | Confirm / Select / Download |
| Space | Toggle selection / Toggle filter mode |
| a | Select all / Deselect all |
| f | Open filter (from conversation list) |
| Tab | Switch filter field / Show/hide token |
| Esc | Go back one step |
| q | Quit |

### Getting Tokens

**ChatGPT Access Token:**
1. Open chatgpt.com
2. F12 → Network tab
3. Find any request → Copy the `Authorization` header value (remove `Bearer ` prefix)

**Claude Session Key:**
1. Open claude.ai
2. F12 → Application → Cookies
3. Copy the `sessionKey` value

**Claude Code / Cursor:** No token needed — reads local session files directly.

## Export

Files are saved as Markdown to:

```
<binary_location>/llm-exports/<platform>/
```

For example: `./llm-exports/chatgpt/My Conversation.md`

## Build

```bash
# macOS
cargo build --release

# Windows (cross-compile from macOS)
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```

---

# LLM Chat Export

LLM 对话导出工具 — 终端 (TUI) 应用，将大模型对话记录导出为本地 Markdown 文件。

支持多平台、关键词和日期筛选、中英文双语界面。

## 支持平台

| 平台 | 认证方式 | 数据来源 |
|------|---------|---------|
| ChatGPT | Access Token | OpenAI API |
| Claude Web | Session Key | Claude.ai API |
| Claude Code | 无需认证 (本地) | `~/.claude/projects/` JSONL 文件 |
| Cursor | 无需认证 (本地) | `~/Library/Application Support/Cursor/` SQLite 数据库 |

## 功能

- **多平台支持** — ChatGPT、Claude Web、Claude Code、Cursor
- **关键词筛选** — 包含 / 不包含模式，逗号分隔，匹配对话标题
- **日期范围筛选** — 开始日期和结束日期 (YYYY-MM-DD)，可选填
- **批量导出** — 空格选中、`a` 全选、Enter 下载
- **双语界面** — 中文和英文，启动时选择
- **逐步返回** — Esc 返回上一步，而不是跳到开头
- **导出后无需重启** — 导出完成后按 Enter 返回，继续导出其他对话
- **跨平台** — macOS 和 Windows

## 使用方法

在终端中运行：

```bash
./llm-chat-export
```

### 操作流程

```
语言选择 → 平台选择 → [Token 输入] → 筛选设置 → 加载 → 会话列表 → 下载中 → 完成
                                                          ↑ f: 重新筛选    ↑ Enter: 返回
```

### 快捷键

| 按键 | 操作 |
|------|-----|
| 上 / 下 / j / k | 导航 |
| Enter | 确认 / 选择 / 下载 |
| 空格 | 切换选中 / 切换筛选模式 |
| a | 全选 / 取消全选 |
| f | 打开筛选 (会话列表中) |
| Tab | 切换筛选字段 / 显示隐藏 Token |
| Esc | 返回上一步 |
| q | 退出 |

### 获取 Token

**ChatGPT Access Token：**
1. 打开 chatgpt.com
2. F12 → Network 标签
3. 找到任意请求 → 复制 `Authorization` 的值 (去掉 `Bearer ` 前缀)

**Claude Session Key：**
1. 打开 claude.ai
2. F12 → Application → Cookies
3. 复制 `sessionKey` 的值

**Claude Code / Cursor：** 无需 Token，直接读取本地会话文件。

## 导出位置

文件以 Markdown 格式保存到：

```
<二进制文件所在目录>/llm-exports/<平台>/
```

例如：`./llm-exports/chatgpt/我的对话.md`

## 构建

```bash
# macOS
cargo build --release

# Windows (从 macOS 交叉编译)
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```
