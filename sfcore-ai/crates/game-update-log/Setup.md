# Setup Instructions - Game Update Log

## 📁 Struktur File

Semua file yang sudah gua kasih harus diletakkan di struktur ini:

```
workspace/
├── Cargo.toml                           # Update workspace members
├── .env                                 # Add DATABASE_URL
└── crates/
    └── game-update-log/                 # ← Folder project baru
        ├── Cargo.toml                   # File #1
        ├── .gitignore                   # File #8 (rename dari .txt)
        ├── README.md                    # File #7
        └── src/
            ├── main.rs                  # File #6
            ├── lib.rs                   # File #5
            ├── models.rs                # File #2
            ├── repository.rs            # File #3a + #3b (gabung jadi 1 file)
            └── scraper.rs               # File #4a + #4b + #4c (gabung jadi 1 file)
```

## 🔨 Step-by-Step Setup

### 1. Buat Folder Project

```bash
cd /path/to/workspace
mkdir -p crates/game-update-log/src
```

### 2. Copy File-File

```bash
# Copy Cargo.toml
cp 1-Cargo.toml crates/game-update-log/Cargo.toml

# Copy README
cp 7-README.md crates/game-update-log/README.md

# Copy .gitignore (rename dari .txt)
cp 8-gitignore.txt crates/game-update-log/.gitignore

# Copy source files
cp 2-models.rs crates/game-update-log/src/models.rs
cp 5-lib.rs crates/game-update-log/src/lib.rs
cp 6-main.rs crates/game-update-log/src/main.rs
```

### 3. Gabung File repository.rs

```bash
# Gabung 3a dan 3b jadi satu file
cat 3a-repository-part1.rs > crates/game-update-log/src/repository.rs
cat 3b-repository-part2.rs >> crates/game-update-log/src/repository.rs
```

### 4. Gabung File scraper.rs

```bash
# Gabung 4a, 4b, dan 4c jadi satu file
cat 4a-scraper-part1.rs > crates/game-update-log/src/scraper.rs
cat 4b-scraper-part2.rs >> crates/game-update-log/src/scraper.rs
cat 4c-scraper-part3.rs >> crates/game-update-log/src/scraper.rs
```

### 5. Update Workspace Cargo.toml

Edit `Cargo.toml` di root workspace, tambahkan:

```toml
[workspace]
members = [
    # ... existing crates
    "crates/game-update-log",  # ← Tambah ini
]

[workspace.dependencies]
# Pastikan ada ini (kemungkinan sudah ada):
clap = { version = "4.5", features = ["derive"] }
regex = "1.10"
```

### 6. Update .env File

Edit `.env` di root workspace:

```dotenv
# Database
DATABASE_URL=/home/sfcore/server-db/gamesmatrix.db

# Logging
RUST_LOG=game_update_log=info,sqlx=warn
```

### 7. Install Playwright

```bash
# Install playwright CLI (via npm)
npm install -g playwright

# Or via cargo
cargo install playwright-cli

# Install chromium browser
playwright install chromium
```

### 8. Build Project

```bash
cd crates/game-update-log
cargo build --release
```

### 9. Test Run

```bash
# Test single update
cargo run --release -- update

# Test validation
cargo run --release -- validate
```

## ✅ Verifikasi

Setelah setup, cek apakah:

- [ ] Folder structure benar
- [ ] Semua file di tempat yang tepas
- [ ] `cargo build` sukses
- [ ] Playwright terinstall
- [ ] Database path di .env benar
- [ ] Test run berhasil

## 🚀 Running

```bash
# Dari workspace root
cargo run --package game-update-log --release -- run

# Atau dari folder project
cd crates/game-update-log
cargo run --release -- run
```

## 📝 Quick Command Reference

```bash
# Build
cargo build --release

# Run continuous
cargo run --release -- run

# Single update
cargo run --release -- update

# Validate logs
cargo run --release -- validate

# Fix missing logs
cargo run --release -- correct

# View logs (if using systemd)
journalctl -u game-update-log -f
```

## 🆘 Common Issues

### Error: "failed to load manifest"

- Cek Cargo.toml syntax
- Pastikan di folder yang benar

### Error: "DATABASE_URL not set"

- Tambahkan ke .env file
- Jalankan dari workspace root

### Error: "playwright browser not found"

```bash
playwright install chromium
```

### Build Error

```bash
cargo clean
cargo build --release
```

## 📞 Next Steps

1. ✅ Setup complete
2. Test dengan single update
3. Monitor logs
4. Setup systemd service (optional)
5. Deploy production
