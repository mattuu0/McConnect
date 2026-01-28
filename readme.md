承知いたしました。
「パケット解析をせず、純粋に **TCP ↔ WebSocket** を中継するプロキシ」という極めてシンプルで強力なコア機能に絞り、それを操作するための CLI ツールをセットにしたプロジェクト構成で `README.md` をまとめました。

---

# McConnect

Rust で構築された、Minecraft 通信のための高性能 WebSocket トンネリング・プロキシ。

## 📋 プロジェクト概要

本プロジェクトは、Minecraft の TCP 通信を WebSocket 上に透過的に流すためのネットワーク・ユーティリティです。
Minecraft クライアント側（Localhost）の TCP ストリームを WebSocket にカプセル化して転送し、リモートサーバー側で再び TCP に戻すことで、ネットワーク制限を回避した接続や独自の通信経路の構築を可能にします。

パケットの内容を解析しない「L4 プロキシ」として動作するため、Minecraft のバージョンに依存せず、極めて低遅延な転送を実現します。

## 🏗 プロジェクト構成

本リポジトリは、再利用可能なコア機能（Library）と、それを即座に利用するためのコマンドラインツール（CLI）で構成されます。

* **mc-connect-core (Library):**
* TCP リスナーと WebSocket クライアント間の双方向ブリッジロジック。
* `tokio` ベースの非同期 I/O による高スループットなデータ転送。


* **mc-connect-cli (Binary):**
* ローカルポート、接続先 WebSocket URL などを指定してトンネルを起動する CLI ツール。



## ✨ 特徴 (Features)

* **透過的トンネリング:** マイクラ側は `localhost:25565` に接続するだけで、背後の WebSocket 通信を意識する必要はありません。
* **プロトコル・フリー:** パケット解析（Deserialization）を行わないため、あらゆる Minecraft バージョン、および他の TCP プロトコルでも利用可能。
* **メモリ安全性と速度:** Rust の所有権モデルと `tokio` の非同期処理により、安全かつオーバーヘッドの少ない中継を実現。
* **シンプルなインターフェース:** ライブラリとして独立しているため、将来的にカスタムランチャー等へ組み込むことが容易。

## 📂 ディレクトリ構成 (Workspace)

```text
.
├── mc-connect-core/      # トンネリングの基幹ロジック (lib)
│   ├── src/
│   │   ├── lib.rs       # 外部向けインターフェース
│   │   ├── tcp.rs       # TCP 接続の受付・管理
│   │   ├── ws.rs        # WebSocket 接続・カプセル化
│   │   └── bridge.rs    # 双方向ストリーム中継 (Copy Bidirectional)
├── mc-connect-cli/       # CLI 実行バイナリ (bin)
│   ├── src/
│   │   └── main.rs      # 引数解析、コアの起動制御
└── Cargo.toml           # ワークスペース管理

```

## 🛠 技術構成 (Tech Stack)

* **Language:** Rust
* **Async Runtime:** `tokio`
* **WebSocket:** `tokio-tungstenite`
* **CLI Parser:** `clap` (予定)

## 🗺 開発ロードマップ

* [ ] **Phase 1: Core Bridge Implementation**
* `tokio::io::copy_bidirectional` 相当のロジックを WebSocket ストリームに適用。


* [ ] **Phase 2: CLI Tooling**
* 接続先やポートを柔軟に指定できる CLI インターフェースの構築。


* [ ] **Phase 3: Robustness & Logging**
* 接続断絶時の再試行ロジック、および通信状況のモニタリング機能。



## 🤝 貢献 (Contribution)

現在は Rust によるネットワークプログラミングの学習および実用的なプロキシツールの構築を目的としています。設計へのアドバイスや最適化に関する提案は大歓迎です。
