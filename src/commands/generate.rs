//! 引数なし: パスワードを生成する（保存しない）

pub fn run(length: usize, symbols: bool, no_copy: bool) {
    super::generate_and_show(length, symbols, no_copy);
}
