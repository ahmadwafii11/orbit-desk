## Jalankan di termux:
sudo apt update

sudo apt install \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

## Periksa Instalasi Node.js, pnpm, dan Rust
node -v \
pnpm -v \
rustc --version \
cargo --version

## Membuat proyek Tauri
pnpm create tauri-app

konfigurasi project:

Pertanyaan | Pilihan
---------- | --------
Project name | orbit-desk
Identifier | com.companion.desktop
Choose which language to use for your frontend | TypeScript / JavaScript - (pnpm, yarn, npm, deno, bun)
Choose your package manager | pnpm
Choose your UI template | React - (https://react.dev/)
Choose your UI flavor | TypeScript 

## Instalasies Dependencies
pnpm add tailwindcss @tailwindcss/vite \
pnpm add lucide-react \
pnpm add react-router-dom

## Menjalankan Tauri App
pnpm tauri dev

## Membuat Tauri App
pnpm tauri build