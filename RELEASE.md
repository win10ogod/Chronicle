# Release packaging

Chronicle 發行版打包腳本位於 `scripts/`，會產生可直接丟到 GitHub Release 的壓縮檔與 SHA256。

## 產物

輸出到 `dist/`：

- Linux/WSL：`chronicle-v<version>-<target>.tar.gz` + `.sha256`
- Windows：`chronicle-v<version>-<target>.zip` + `.sha256`

壓縮檔內容包含：

- `chronicle` / `chronicle.exe`
- `README.md`
- `completions/`（bash/zsh/fish/powershell/elvish）

## 打包（WSL / Linux）

```bash
bash scripts/package.sh
```

若要跳過 fmt/clippy/test：

```bash
bash scripts/package.sh --no-verify
```

## 打包（Windows PowerShell）

```powershell
.\scripts\package.ps1
```

跳過驗證：

```powershell
.\scripts\package.ps1 -NoVerify
```

## 一次打包兩邊（從 WSL 呼叫 Windows）

```bash
bash scripts/package-all.sh
```

