//! genpasswd_ex — パスワード生成 + サービスごとの履歴管理 CLI
//!
//! main.rs はエントリーポイントで、引数を解析して各サブコマンドの処理
//! （`commands/<名前>.rs` の `run()`）に振り分けるだけにしている。

// CLI 定義（clap derive）
mod cli;
// クリップボードへのコピー（OS ごとに実装を切り替え）
mod clipboard;
// サブコマンドの処理（1 コマンド 1 ファイル）と共通ヘルパー
mod commands;
// パスワードの暗号化/復号と鍵の取得（キーチェーン / 資格情報マネージャー）
mod crypto;
// SQLite アクセス（password_history テーブル）
mod db;
// パスワード生成
mod password;

use clap::Parser;
use cli::{Args, Command};

fn main() {
    // .env があれば読み込む（GENPASSWD_EX_DB / GENPASSWD_EX_KEY を設定できる）
    dotenvy::dotenv().ok();
    let args = Args::parse();

    match args.command {
        // サブコマンドなし: パスワードを生成して表示するだけ（保存しない）
        None => commands::generate::run(args.length, args.symbols, args.no_copy),

        // ---- 履歴への登録 ----
        // 生成して保存
        Some(Command::Save { service, username, length, symbols, no_copy }) => {
            commands::save::run(service, username, length, symbols, no_copy)
        }
        // 既存のパスワードを保存（別名 register）
        Some(Command::Add { service, password, username }) => commands::add::run(service, password, username),

        // ---- 参照 ----
        // 履歴表示（-s 省略時は全サービスをマスク表示、-t でタグ絞り込み）
        Some(Command::History { service, tag }) => commands::history::run(service, tag),
        // 指定 ID のパスワードをクリップボードへ
        Some(Command::Copy { id }) => commands::copy::run(id),
        // サービス一覧と件数
        Some(Command::List) => commands::list::run(),

        // ---- 変更 ----
        // ユーザ名・パスワードの更新
        Some(Command::Update { id, username, password }) => commands::update::run(id, username, password),
        // サービス名の変更（--merge で既存サービスに統合）
        Some(Command::Rename { service, new_service, merge }) => commands::rename::run(service, new_service, merge),
        // 旧形式（平文）のパスワードを一括暗号化
        Some(Command::Encrypt) => commands::encrypt::run(),

        // ---- タグ ----
        // サービスにタグを付ける
        Some(Command::Tag { service, tags }) => commands::tag::run(service, tags),
        // サービスからタグを外す
        Some(Command::Untag { service, tags }) => commands::untag::run(service, tags),
        // タグごとのサービス一覧
        Some(Command::Tags) => commands::tags::run(),

        // ---- 削除 ----
        // 1 件削除
        Some(Command::Remove { id }) => commands::remove::run(id),
        // サービスの全履歴を削除
        Some(Command::Delete { service }) => commands::delete::run(service),
    }
}
