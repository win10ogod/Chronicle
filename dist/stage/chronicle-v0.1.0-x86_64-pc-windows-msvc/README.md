# Chronicle

Chronicle 是一個為 AI Agent 設計的「純文字 + Git 版本控制」記憶系統：記憶即檔案（Linked‑Markdown），可讀、可改、可回滾。

2.0 版核心：**MAMA 2.0（SimHash 語義指紋 + ACT‑R 認知熱度衰減 + Spreading Activation）**與 **Shadow Index（`.chronicle/index.json`）**，在不依賴向量資料庫的前提下做快速檢索。

## 快速開始

```bash
# 在你的專案根目錄
cargo build --release

./target/release/chronicle init --git-init
./target/release/chronicle remember -m "在 Windows 下 fd-lock 需注意句柄關閉 [[Rust]] #bug-fix" -t concurrency
./target/release/chronicle recall -q "Rust 併發 鎖" -k 2
```

## 目錄結構

Chronicle 會在專案根目錄建立並維護：

```
.chronicle/
  short_term/   # 工作記憶（高優先、100% 保留率）
  long_term/    # 長期記憶（MAMA 算法檢索 + 遺忘）
  archive/      # 歸檔（低頻/低強度記憶）
  config.yaml   # MAMA 權重與參數
  index.json    # Shadow Index（metadata + SimHash）
  wal/          # 非同步 Git 提交任務佇列（WAL）
  wal.lock
```

## 存儲規範（Frontmatter）

每條記憶是 `.md` 檔，YAML frontmatter 會包含：

- `id`
- `timestamp`
- `last_access`
- `hit_count`
- `simhash`（`0x` + 64-bit hex）
- `tags`
- `links`

## 核心指令

- `chronicle init [path] [--git-init]`
- `chronicle remember -m <msg> [-t <tag>...] [--layer short|long|archive] [--id <id>]`
- `chronicle recall -q <query> [-k <top_k>] [--include-archive] [--json] [--no-touch]`
- `chronicle forget --id <id> | --threshold <val> [--dry-run]`
- `chronicle log [--limit n]`
- `chronicle consolidate [--min-hits N] [--min-age-hours H] [--dry-run]`
- `chronicle branch create|checkout|merge|list|current|delete ...`

## Git 提交模式（Sync/Async）

寫入或更新命中熱度後，Chronicle 會做原子提交。可用全域參數控制：

- `--commit async`：寫入後立刻返回（預設），提交由背景 WAL worker 處理
- `--commit sync`：同步提交（需要確保 `git log` 立即可見時使用）
- `--commit off`：不提交（等同各指令的 `--no-commit`）

## 加密（可選）

若設定 `CHRONICLE_KEY`，Chronicle 會用 AES‑256‑GCM 對磁碟上的記憶檔做靜態加密（檔案仍為 `.md`，但內容為二進位）。

支援三種 key 形式：

- `CHRONICLE_KEY="base64:<32-bytes>"`：直接使用 32 位元組 key
- `CHRONICLE_KEY="hex:<64-hex>"`：直接使用 32 位元組 key
- `CHRONICLE_KEY="<passphrase>"`：以 Argon2id + `.chronicle/crypto.yaml` 內的 salt 推導 per‑repo key

## 發行版打包

見 `RELEASE.md`（`scripts/package.sh` / `scripts/package.ps1`）。
