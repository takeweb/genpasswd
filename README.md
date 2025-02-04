# セキュアなパスワードジェネレーター

## 環境構築
```
cargo add clap --features derive
cargo add rand
cargo add rand_distr
```

## コマンドのインストール
```
cargo build --release
cp target/release/genpasswd ~/bin/
```

## 使い方
```

genpasswd -h
Secure Password Generator

Usage: genpasswd [OPTIONS]

Options:
  -l, --length <LENGTH>  Length of the password [default: 16]
  -s, --symbols          Include symbols in the password
  -h, --help             Print help
  -V, --version          Print version
```

以上