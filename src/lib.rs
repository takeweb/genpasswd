use rand::Rng;
use rand_distr::{Alphanumeric, Distribution}; // rand_distrを使ってAlphanumericをインポート

pub fn generate_password(length: usize, include_symbols: bool) -> String {
    let mut rng = rand::rng();
    let mut password: Vec<char> = (0..length)
        .map(|_| Alphanumeric.sample(&mut rng) as char) // Alphanumericを使ってランダム文字を生成
        .collect();

    if include_symbols {
        let symbols = "!@#$%^&*()_+=-[]{};:,.<>?";
        // 3文字程度をランダムで置き換える
        for c in password.iter_mut().take(3) {
            *c = symbols
                .chars()
                .nth(rng.random_range(0..symbols.len()))
                .unwrap();
        }
    }

    password.into_iter().collect() // Vec<char>をStringに変換
}
