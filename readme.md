# McConnect

Rust で構築された、Minecraft 通信のための高性能 WebSocket トンネリング・プロキシ。

## 📋 プロジェクト概要

本プロジェクトは、Minecraft の TCP 通信を WebSocket 上に透過的に流すためのネットワーク・ユーティリティです。
Minecraft クライアント側（Localhost）の TCP ストリームを WebSocket にカプセル化して転送し、リモートサーバー側で再び TCP に戻すことで、ネットワーク制限を回避した接続や独自の通信経路の構築を可能にします。

パケットの内容を解析しない「L4 プロキシ」として動作するため、Minecraft のバージョンに依存せず、極めて低遅延な転送を実現します。

## 🏗 プロジェクト構成

本リポジトリは、コア機能（Library）、デスクトップアプリ（GUI）、およびコマンドラインツール（CLI）で構成されます。

*   **mc-connect-core (Library):**
    *   TCP リスナーと WebSocket クライアント間の双方向ブリッジロジック。
    *   `tokio` ベースの非同期 I/O による高スループットなデータ転送。
*   **mc-connect-app (Tauri / GUI):**
    *   直感的な操作が可能なデスクトップアプリケーション。
    *   マッピング管理、リアルタイム統計表示、鍵生成機能などを搭載。
*   **mc-connect-cli (Binary):**
    *   軽量なコマンドラインインターフェース。

## ✨ 特徴 (Features)

*   **透過的トンネリング:** マイクラ側は `localhost:25565` に接続するだけで、背後の WebSocket 通信を意識する必要はありません。
*   **プロトコル・フリー:** パケット解析（Deserialization）を行わないため、あらゆる Minecraft バージョン、および他の TCP プロトコルでも利用可能。
*   **メモリ安全性と速度:** Rust の所有権モデルと `tokio` の非同期処理により、安全かつオーバーヘッドの少ない中継を実現。
*   **シンプルなインターフェース:** ライブラリとして独立しているため、カスタムランチャー等へ組み込むことが容易。

## 📂 ディレクトリ構成 (Workspace)

```text
.
├── mc-connect-core/      # トンネリングの基幹ロジック (lib)
├── mc-connect-app/       # GUI アプリケーション (Tauri + React)
├── mc-connect-cli/       # CLI 実行バイナリ (bin)
└── Cargo.toml           # ワークスペース管理
```

## 🛠 技術構成 (Tech Stack)

*   **Language:** Rust
*   **Backend Support:** `tauri`, `actix-web`, `tokio`
*   **Frontend:** `React`, `Vite`, `TailwindCSS` (or Vanilla CSS), `Framer Motion`
*   **WebSocket:** `tokio-tungstenite`
*   **CLI Parser:** `clap`

## 🤝 貢献 (Contribution)

現在は Rust によるネットワークプログラミングの学習および実用的なプロキシツールの構築を目的としています。設計へのアドバイスや最適化に関する提案は大歓迎です。

## 📄 ライセンス (License)

### ソフトウェア本体
本ソフトウェアは **MIT License** の下で公開されています。

### 使用ライブラリ
本プロジェクトでは、以下の主要なオープンソースライブラリを使用しています。各ライブラリのライセンスについては、それぞれの公式ドキュメントを参照してください。

*   **Rust エコシステム:** `tokio`, `actix-web`, `serde`, `tauri` 等 (MIT / Apache-2.0)
*   **フロントエンド:** `React` (MIT), `Vite` (MIT), `Lucide React` (ISC), `Framer Motion` (MIT)

### アプリアイコン
本アプリケーションで使用しているアイコンは、以下のサイトより取得したものを改変して使用しています。

*   **提供元:** [icon-icons.com - Minecraft Icon](https://icon-icons.com/ja/icon/minecraft/23386)
*   **ライセンス:** [Creative Commons (BY-NC-ND 4.0)](https://creativecommons.org/licenses/by-nc-nd/4.0/)
