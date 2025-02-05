# セキュアなパスワードジェネレーター

## 環境構築
```
cargo add clap --features derive
cargo add rand
cargo add rand_distr
```

## コマンドのインストール crate.ioに同名のコマンドが存在する為、カレントパスをインストールするように指定
```
cargo install --path .
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