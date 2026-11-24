# FokusFlow Desktop

Ini adalah shell Tauri ringan untuk membuat FokusFlow menjadi aplikasi Windows asli, bukan tab browser.

## Build Windows

Jalankan di Windows 10/11 yang telah memiliki Node.js LTS, Rust, dan Microsoft C++ Build Tools:

```bash
npm install
npm run build:windows
```

Installer MSI akan keluar di `native/src-tauri/target/release/bundle/msi/`.

Data aplikasi tetap disimpan lokal pada perangkat. Fitur blokir aplikasi dan situs tingkat sistem akan ditambahkan di tahap berikutnya karena membutuhkan izin Windows dan aturan yang ditetapkan pengguna.
