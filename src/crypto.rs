use aes_gcm::aead::{Aead, Generate, Key, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;

/// 暗号化済みの値に付けるプレフィックス（これがない値は旧形式の平文として扱う）
const PREFIX: &str = "enc:v1:";
const NONCE_LEN: usize = 12;

/// 鍵ストアに鍵を保存するときのサービス名
const KEYCHAIN_SERVICE: &str = "genpasswd";
/// 旧ツール名（genpasswd_ex）時代のサービス名。新しい名前で鍵が見つからないときに引き継ぎ元として使う
const LEGACY_KEYCHAIN_SERVICE: &str = "genpasswd_ex";

/// キーチェーンのアカウント名（debug ビルドは本番の鍵と分ける）
fn keychain_account() -> &'static str {
    if cfg!(debug_assertions) { "db-key-dev" } else { "db-key" }
}

/// 環境変数で指定された鍵（`GENPASSWD_KEY`、旧名 `GENPASSWD_EX_KEY` も可）
fn key_from_env() -> Option<(String, &'static str)> {
    ["GENPASSWD_KEY", "GENPASSWD_EX_KEY"]
        .into_iter()
        .find_map(|name| std::env::var(name).ok().map(|v| (v, name)))
}

/// 鍵ストアから鍵を取得する。なければ旧名の鍵を新しい名前に複写して使い、それもなければ生成して登録する
fn load_or_create_key() -> Result<Vec<u8>, String> {
    let account = keychain_account();
    if let Some(key) = keychain::load(KEYCHAIN_SERVICE, account)? {
        return Ok(key);
    }
    // 旧名の鍵は削除せずに残す（旧バージョンのバイナリでも引き続き使えるように）
    if let Some(key) = keychain::load(LEGACY_KEYCHAIN_SERVICE, account)? {
        keychain::store(KEYCHAIN_SERVICE, account, &key)?;
        eprintln!(
            "Copied encryption key from {} to {} in {} (account {}).",
            LEGACY_KEYCHAIN_SERVICE,
            KEYCHAIN_SERVICE,
            keychain::STORE_NAME,
            account
        );
        return Ok(key);
    }
    let key = Key::<Aes256Gcm>::generate().to_vec();
    keychain::store(KEYCHAIN_SERVICE, account, &key)?;
    eprintln!("Created encryption key in {} ({} / {}).", keychain::STORE_NAME, KEYCHAIN_SERVICE, account);
    Ok(key)
}

pub struct Cipher {
    aead: Aes256Gcm,
}

impl Cipher {
    /// 鍵を読み込む。環境変数 `GENPASSWD_KEY`（base64）があれば優先し、
    /// なければ OS の鍵ストア（macOS: キーチェーン / Windows: 資格情報マネージャー）から取得する
    /// （未登録なら旧名の鍵を引き継ぐか、生成して登録する）
    pub fn load() -> Result<Self, String> {
        let key = match key_from_env() {
            Some((b64, name)) => B64
                .decode(b64.trim())
                .map_err(|e| format!("invalid {}: {}", name, e))?,
            None => load_or_create_key()?,
        };
        let key = Key::<Aes256Gcm>::try_from(key.as_slice())
            .map_err(|_| format!("invalid key length: {} bytes (expected 32)", key.len()))?;
        Ok(Self { aead: Aes256Gcm::new(&key) })
    }

    pub fn encrypt(&self, plaintext: &str) -> String {
        let nonce = Nonce::generate();
        let ciphertext = self
            .aead
            .encrypt(&nonce, plaintext.as_bytes())
            .expect("AES-GCM encryption failed");
        let mut buf = nonce.to_vec();
        buf.extend_from_slice(&ciphertext);
        format!("{}{}", PREFIX, B64.encode(buf))
    }

    /// 暗号化済みの値を復号する。プレフィックスがない値（旧形式の平文）はそのまま返す
    pub fn decrypt(&self, stored: &str) -> Result<String, String> {
        let Some(b64) = stored.strip_prefix(PREFIX) else {
            return Ok(stored.to_string());
        };
        let buf = B64.decode(b64).map_err(|e| format!("invalid base64: {}", e))?;
        if buf.len() < NONCE_LEN {
            return Err("ciphertext too short".to_string());
        }
        let (nonce, ciphertext) = buf.split_at(NONCE_LEN);
        let nonce = Nonce::try_from(nonce).expect("nonce length checked");
        let plaintext = self
            .aead
            .decrypt(&nonce, ciphertext)
            .map_err(|_| "decryption failed (wrong key?)".to_string())?;
        String::from_utf8(plaintext).map_err(|e| format!("invalid UTF-8: {}", e))
    }
}

pub fn is_encrypted(stored: &str) -> bool {
    stored.starts_with(PREFIX)
}

/// macOS のキーチェーン（汎用パスワード、サービス名 / アカウント名）に鍵を保存する
#[cfg(target_os = "macos")]
mod keychain {
    use security_framework::passwords::{get_generic_password, set_generic_password};
    use security_framework_sys::base::errSecItemNotFound;

    pub const STORE_NAME: &str = "keychain";

    /// 鍵を読み込む（未登録は None。拒否などのエラーは Err にして、既存の鍵を上書きしないようにする）
    pub fn load(service: &str, account: &str) -> Result<Option<Vec<u8>>, String> {
        match get_generic_password(service, account) {
            Ok(key) => Ok(Some(key)),
            Err(e) if e.code() == errSecItemNotFound => Ok(None),
            Err(e) => Err(format!("failed to read key from keychain: {}", e)),
        }
    }

    pub fn store(service: &str, account: &str, key: &[u8]) -> Result<(), String> {
        set_generic_password(service, account, key).map_err(|e| format!("failed to save key to keychain: {}", e))
    }
}

/// Windows の資格情報マネージャー（汎用資格情報、ターゲット名 `<service>:<account>`）に鍵を保存する
#[cfg(windows)]
mod keychain {
    use std::ptr;
    use windows_sys::Win32::Foundation::{ERROR_NOT_FOUND, GetLastError};
    use windows_sys::Win32::Security::Credentials::{
        CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredFree, CredReadW, CredWriteW,
    };

    pub const STORE_NAME: &str = "Windows Credential Manager";

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// 鍵を読み込む（未登録は None。その他のエラーは Err にして、既存の鍵を上書きしないようにする）
    pub fn load(service: &str, account: &str) -> Result<Option<Vec<u8>>, String> {
        let target_w = wide(&format!("{}:{}", service, account));
        let mut cred: *mut CREDENTIALW = ptr::null_mut();
        // SAFETY: target_w は NUL 終端の UTF-16。成功時の cred は CredFree で解放する
        let ok = unsafe { CredReadW(target_w.as_ptr(), CRED_TYPE_GENERIC, 0, &mut cred) };
        if ok != 0 {
            let key = unsafe {
                let c = &*cred;
                let blob = std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize).to_vec();
                CredFree(cred as *const _);
                blob
            };
            return Ok(Some(key));
        }
        let err = unsafe { GetLastError() };
        if err == ERROR_NOT_FOUND {
            Ok(None)
        } else {
            Err(format!("failed to read key from credential manager (error {})", err))
        }
    }

    pub fn store(service: &str, account: &str, key: &[u8]) -> Result<(), String> {
        let mut target_w = wide(&format!("{}:{}", service, account));
        let mut user_w = wide(account);
        let mut blob = key.to_vec();
        let cred = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target_w.as_mut_ptr(),
            Comment: ptr::null_mut(),
            LastWritten: unsafe { std::mem::zeroed() },
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: blob.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: ptr::null_mut(),
            TargetAlias: ptr::null_mut(),
            UserName: user_w.as_mut_ptr(),
        };
        // SAFETY: cred が参照するバッファはこの呼び出しの間生きている
        if unsafe { CredWriteW(&cred, 0) } == 0 {
            let err = unsafe { GetLastError() };
            return Err(format!("failed to save key to credential manager (error {})", err));
        }
        Ok(())
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod keychain {
    pub const STORE_NAME: &str = "key store";

    const UNSUPPORTED: &str = "keychain is only supported on macOS and Windows; set GENPASSWD_KEY instead";

    pub fn load(_service: &str, _account: &str) -> Result<Option<Vec<u8>>, String> {
        Err(UNSUPPORTED.to_string())
    }

    pub fn store(_service: &str, _account: &str, _key: &[u8]) -> Result<(), String> {
        Err(UNSUPPORTED.to_string())
    }
}
