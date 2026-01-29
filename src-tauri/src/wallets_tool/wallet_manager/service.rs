use anyhow::Result;
use sqlx::{SqlitePool, Row};
use chrono::Utc;
use std::sync::Mutex;
use rand::Rng;
use sha2::Sha256;
use hmac::Hmac;
use hmac::Mac;
use pbkdf2::pbkdf2;
use aes::Aes256;
use cbc::{Encryptor, Decryptor};
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use alloy_signer_local::{PrivateKeySigner, MnemonicBuilder, coins_bip39::{English, Mnemonic}};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer as _;
use sha2::Sha512;
use super::models::*;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;
type HmacSha512 = Hmac<Sha512>;

pub struct WalletManagerService {
    pool: SqlitePool,
    // Cached Master Data Key (decrypted)
    mdk: Mutex<Option<[u8; 32]>>,
}

impl WalletManagerService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            mdk: Mutex::new(None),
        }
    }

    /// Initialize tables if they don't exist
    pub async fn init_tables(&self) -> Result<()> {
        // Wallet Groups
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                parent_id INTEGER,
                name TEXT NOT NULL,
                chain_type TEXT, -- Added for ecosystem isolation
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (parent_id) REFERENCES wallet_groups(id) ON DELETE CASCADE
            )
            "#
        ).execute(&self.pool).await?;

        // Migration: Check if chain_type column exists, if not add it
        // This is a simple migration check. For production, better use sqlx migrations.
        // But for this project scale, this is fine.
        let check_col = sqlx::query("SELECT chain_type FROM wallet_groups LIMIT 1")
            .fetch_optional(&self.pool)
            .await;
        
        if check_col.is_err() {
            // Column likely doesn't exist
            let _ = sqlx::query("ALTER TABLE wallet_groups ADD COLUMN chain_type TEXT").execute(&self.pool).await;
        }

        // Wallets
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id INTEGER,
                name TEXT,
                address TEXT NOT NULL,
                chain_type TEXT NOT NULL, -- 'evm' or 'solana'
                encrypted_private_key TEXT,
                encrypted_mnemonic TEXT,
                mnemonic_index INTEGER,
                remark TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (group_id) REFERENCES wallet_groups(id) ON DELETE SET NULL
            )
            "#
        ).execute(&self.pool).await?;

        // Add remark column if not exists (migration)
        let check_col = sqlx::query("SELECT remark FROM wallets LIMIT 1")
            .fetch_optional(&self.pool)
            .await;
        if check_col.is_err() {
            let _ = sqlx::query("ALTER TABLE wallets ADD COLUMN remark TEXT")
                .execute(&self.pool)
                .await;
        }

        let check_col = sqlx::query("SELECT mnemonic_index FROM wallets LIMIT 1")
            .fetch_optional(&self.pool)
            .await;
        if check_col.is_err() {
            let _ = sqlx::query("ALTER TABLE wallets ADD COLUMN mnemonic_index INTEGER")
                .execute(&self.pool)
                .await;
        }

        // App Config (for password hash/salt and encrypted MDK)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#
        ).execute(&self.pool).await?;

        Ok(())
    }

    /// Check if master password is set
    pub async fn is_password_set(&self) -> Result<bool> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM app_config WHERE key = 'master_verifier'"
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    /// Initialize master password
    pub async fn init_password(&self, password: &str) -> Result<()> {
        if self.is_password_set().await? {
            return Err(anyhow::anyhow!("密码已设置"));
        }

        // 1. Generate Master Data Key (MDK)
        let mut mdk = [0u8; 32];
        rand::thread_rng().fill(&mut mdk);

        // 2. Generate Verifier (for password check)
        let mut verifier_salt = [0u8; 16];
        rand::thread_rng().fill(&mut verifier_salt);
        let verifier_hash = self.derive_key(password, &verifier_salt); // 32 bytes

        // Store verifier: salt(hex) + hash(hex)
        let verifier_str = format!("{}:{}", hex::encode(verifier_salt), hex::encode(verifier_hash));
        self.set_config("master_verifier", &verifier_str).await?;

        // 3. Encrypt MDK with Password-Derived Key (KEK)
        let mut kek_salt = [0u8; 16];
        rand::thread_rng().fill(&mut kek_salt);
        let kek = self.derive_key(password, &kek_salt);

        let (encrypted_mdk, iv) = self.encrypt_data(&mdk, &kek)?;
        
        // Store MDK: salt(hex) + iv(hex) + ciphertext(base64)
        let mdk_str = format!("{}:{}:{}", hex::encode(kek_salt), hex::encode(iv), encrypted_mdk);
        self.set_config("master_key", &mdk_str).await?;

        // Cache MDK
        *self.mdk.lock().unwrap() = Some(mdk);

        Ok(())
    }

    /// Verify password and unlock MDK
    pub async fn unlock(&self, password: &str) -> Result<bool> {
        // 1. Check Verifier
        let verifier_str = self.get_config("master_verifier").await?
            .ok_or_else(|| anyhow::anyhow!("尚未初始化密码"))?;
        
        let parts: Vec<&str> = verifier_str.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Verifier format error"));
        }
        let salt = hex::decode(parts[0])?;
        let stored_hash = hex::decode(parts[1])?;

        let derived_hash = self.derive_key(password, &salt);
        if derived_hash.as_slice() != stored_hash.as_slice() {
            return Ok(false);
        }

        // 2. Decrypt MDK
        let mdk_str = self.get_config("master_key").await?
            .ok_or_else(|| anyhow::anyhow!("Master Key missing"))?;
        
        let parts: Vec<&str> = mdk_str.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Master Key format error"));
        }
        let kek_salt = hex::decode(parts[0])?;
        let kek = self.derive_key(password, &kek_salt);
        
        let iv = hex::decode(parts[1])?;
        let encrypted_mdk = parts[2]; // base64

        let mdk_bytes = self.decrypt_data(encrypted_mdk, &kek, &iv)?;
        if mdk_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid MDK length"));
        }

        let mut mdk_arr = [0u8; 32];
        mdk_arr.copy_from_slice(&mdk_bytes);

        *self.mdk.lock().unwrap() = Some(mdk_arr);
        self.migrate_plaintext_wallet_secrets(&mdk_arr).await?;

        Ok(true)
    }

    /// Change master password (re-encrypt MDK)
    pub async fn change_password(&self, old_password: &str, new_password: &str) -> Result<()> {
        if !self.unlock(old_password).await? {
            return Err(anyhow::anyhow!("旧密码错误"));
        }

        let mdk = {
            let mdk_guard = self.mdk.lock().unwrap();
            mdk_guard.clone().ok_or_else(|| anyhow::anyhow!("MDK not unlocked"))?
        };

        // 1. New Verifier
        let mut verifier_salt = [0u8; 16];
        rand::thread_rng().fill(&mut verifier_salt);
        let verifier_hash = self.derive_key(new_password, &verifier_salt);
        let verifier_str = format!("{}:{}", hex::encode(verifier_salt), hex::encode(verifier_hash));
        self.set_config("master_verifier", &verifier_str).await?;

        // 2. Re-encrypt MDK
        let mut kek_salt = [0u8; 16];
        rand::thread_rng().fill(&mut kek_salt);
        let kek = self.derive_key(new_password, &kek_salt);
        let (encrypted_mdk, iv) = self.encrypt_data(&mdk, &kek)?;
        let mdk_str = format!("{}:{}:{}", hex::encode(kek_salt), hex::encode(iv), encrypted_mdk);
        self.set_config("master_key", &mdk_str).await?;

        Ok(())
    }

    /// Create Group
    pub async fn create_group(&self, request: CreateGroupRequest) -> Result<i64> {
        // Check for duplicate name in the SAME chain_type (ecosystem)
        // If parent_id is present, it's a sub-group.
        // The requirement says: "Limit duplicate names in the same ecosystem".
        // It implies globally unique names per ecosystem OR unique siblings?
        // Usually file systems allow same name in different folders.
        // But "different ecosystems can have same name" suggests ecosystem-wide uniqueness or at least root level?
        // Let's assume the user means "sibling uniqueness" or "global uniqueness per chain"?
        // Re-reading: "limit same name groups under same ecosystem".
        // Let's enforce uniqueness within the ecosystem for simplicity and safety as requested,
        // or check if it just means "sibling name conflict".
        // Given "different ecosystems can have same name", it strongly implies the name scope is the Ecosystem.
        // So we will check if ANY group in this chain_type has the same name.
        
        // Wait, standard tree logic usually allows same name in different subfolders.
        // But if the user explicitly mentioned this rule, they likely want to avoid confusion.
        // However, strict global uniqueness is harsh. 
        // Let's interpret "limit same name group under same ecosystem" as:
        // You cannot have two groups named "A" in "EVM" ecosystem, regardless of nesting?
        // OR just "sibling" uniqueness?
        // User said: "Limit same name groups under the same ecosystem, different ecosystem can have same name".
        // This sounds like (Name, ChainType) must be unique.
        
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM wallet_groups WHERE name = ? AND chain_type = ?"
        )
        .bind(&request.name)
        .bind(&request.chain_type)
        .fetch_one(&self.pool)
        .await?;

        if count > 0 {
            return Err(anyhow::anyhow!("该生态下已存在同名分组"));
        }

        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO wallet_groups (parent_id, name, chain_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?) RETURNING id"
        )
        .bind(request.parent_id)
        .bind(request.name)
        .bind(request.chain_type)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    /// Get Groups
    pub async fn get_groups(&self) -> Result<Vec<WalletGroup>> {
        let groups = sqlx::query_as::<_, WalletGroup>(
            "SELECT * FROM wallet_groups ORDER BY created_at"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(groups)
    }

    /// Create Wallet
    pub async fn create_wallet(&self, request: CreateWalletRequest) -> Result<i64> {
        let mode = match (
            request.sealed_mnemonic.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false),
            request.sealed_private_key.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false),
        ) {
            (true, false) => CreateWalletsMode::MnemonicImport,
            (false, true) => CreateWalletsMode::PrivateKeyImport,
            (true, true) => return Err(anyhow::anyhow!("助记词与私钥只能二选一")),
            (false, false) => return Err(anyhow::anyhow!("缺少助记词或私钥")),
        };

        let created = self.create_wallets(CreateWalletsRequest {
            group_id: request.group_id,
            name: request.name,
            chain_type: request.chain_type,
            mode,
            sealed_mnemonic: request.sealed_mnemonic,
            sealed_private_key: request.sealed_private_key,
            count: 1,
            start_index: Some(0),
            word_count: None,
            remark: request.remark,
            password: request.password,
        }, request.address).await?;

        created.first().map(|w| w.id).ok_or_else(|| anyhow::anyhow!("创建失败"))
    }

    pub async fn create_wallets(&self, request: CreateWalletsRequest, address_override: Option<String>) -> Result<Vec<WalletInfo>> {
        if !self.unlock(&request.password).await? {
            return Err(anyhow::anyhow!("密码错误"));
        }

        if request.count == 0 {
            return Err(anyhow::anyhow!("钱包数量必须大于0"));
        }

        if request.count > 100 {
            return Err(anyhow::anyhow!("单次最多创建100个钱包"));
        }

        let mdk = {
            let mdk_guard = self.mdk.lock().unwrap();
            mdk_guard.clone().ok_or_else(|| anyhow::anyhow!("未解锁"))?
        };

        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        match request.mode {
            CreateWalletsMode::PrivateKeyImport => {
                let sealed_private_key = request.sealed_private_key.clone().unwrap_or_default().trim().to_string();
                if sealed_private_key.is_empty() {
                    return Err(anyhow::anyhow!("私钥不能为空"));
                }
                if request.sealed_mnemonic.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false) {
                    return Err(anyhow::anyhow!("助记词与私钥只能二选一"));
                }

                let private_key = self.open_sealed_secret(&sealed_private_key, &request.password)?;
                let (address, normalized_private_key) = self.derive_address_from_private_key(&request.chain_type, &private_key)?;
                if let Some(addr) = address_override.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
                    if addr != address {
                        return Err(anyhow::anyhow!("地址与私钥计算结果不一致"));
                    }
                }

                let name = request.name.clone();
                let remark = request.remark.clone();
                let id = self.insert_wallet_row(&mut tx, &mdk, request.group_id, name.clone(), &address, &request.chain_type, Some(&normalized_private_key), None, None, remark.clone()).await?;
                let sealed_private_key_out = self.seal_secret_for_transport(&normalized_private_key, &request.password)?;
                created.push(WalletInfo {
                    id,
                    group_id: request.group_id,
                    name,
                    address,
                    chain_type: request.chain_type,
                    has_private_key: true,
                    has_mnemonic: false,
                    sealed_private_key: Some(sealed_private_key_out),
                    sealed_mnemonic: None,
                    mnemonic_index: None,
                    remark,
                });
            }
            CreateWalletsMode::MnemonicImport => {
                let sealed_mnemonic = request.sealed_mnemonic.clone().unwrap_or_default().trim().to_string();
                if sealed_mnemonic.is_empty() {
                    return Err(anyhow::anyhow!("助记词不能为空"));
                }
                if request.sealed_private_key.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false) {
                    return Err(anyhow::anyhow!("助记词与私钥只能二选一"));
                }

                let mnemonic = self.open_sealed_secret(&sealed_mnemonic, &request.password)?;
                let sealed_mnemonic_out = self.seal_secret_for_transport(&mnemonic, &request.password)?;
                let start_index = request.start_index.unwrap_or(0);
                for i in 0..request.count {
                    let index = start_index + i;
                    let (address, private_key) = self.derive_from_mnemonic(&request.chain_type, &mnemonic, index)?;
                    if let Some(addr) = address_override.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
                        if request.count == 1 && addr != address {
                            return Err(anyhow::anyhow!("地址与助记词计算结果不一致"));
                        }
                    }

                    let name = self.resolve_wallet_name(request.name.as_ref(), request.count, index);
                    let id = self.insert_wallet_row(&mut tx, &mdk, request.group_id, name.clone(), &address, &request.chain_type, Some(&private_key), Some(&mnemonic), Some(index as i64), request.remark.clone()).await?;
                    let sealed_private_key_out = self.seal_secret_for_transport(&private_key, &request.password)?;
                    created.push(WalletInfo {
                        id,
                        group_id: request.group_id,
                        name,
                        address,
                        chain_type: request.chain_type.clone(),
                        has_private_key: true,
                        has_mnemonic: true,
                        sealed_private_key: Some(sealed_private_key_out),
                        sealed_mnemonic: Some(sealed_mnemonic_out.clone()),
                        mnemonic_index: Some(index as i64),
                        remark: request.remark.clone(),
                    });
                }
            }
            CreateWalletsMode::GenerateSameMnemonic => {
                let (mnemonic, _) = self.generate_mnemonic(request.word_count.unwrap_or(12))?;
                let sealed_mnemonic_out = self.seal_secret_for_transport(&mnemonic, &request.password)?;
                let start_index = request.start_index.unwrap_or(0);
                for i in 0..request.count {
                    let index = start_index + i;
                    let (address, private_key) = self.derive_from_mnemonic(&request.chain_type, &mnemonic, index)?;
                    let name = self.resolve_wallet_name(request.name.as_ref(), request.count, index);
                    let id = self.insert_wallet_row(&mut tx, &mdk, request.group_id, name.clone(), &address, &request.chain_type, Some(&private_key), Some(&mnemonic), Some(index as i64), request.remark.clone()).await?;
                    let sealed_private_key_out = self.seal_secret_for_transport(&private_key, &request.password)?;
                    created.push(WalletInfo {
                        id,
                        group_id: request.group_id,
                        name,
                        address,
                        chain_type: request.chain_type.clone(),
                        has_private_key: true,
                        has_mnemonic: true,
                        sealed_private_key: Some(sealed_private_key_out),
                        sealed_mnemonic: Some(sealed_mnemonic_out.clone()),
                        mnemonic_index: Some(index as i64),
                        remark: request.remark.clone(),
                    });
                }
            }
            CreateWalletsMode::GenerateDifferentMnemonic => {
                for i in 0..request.count {
                    let (mnemonic, _) = self.generate_mnemonic(request.word_count.unwrap_or(12))?;
                    let index = 0u32;
                    let (address, private_key) = self.derive_from_mnemonic(&request.chain_type, &mnemonic, index)?;
                    let name = self.resolve_wallet_name(request.name.as_ref(), request.count, i);
                    let id = self.insert_wallet_row(&mut tx, &mdk, request.group_id, name.clone(), &address, &request.chain_type, Some(&private_key), Some(&mnemonic), Some(index as i64), request.remark.clone()).await?;
                    let sealed_private_key_out = self.seal_secret_for_transport(&private_key, &request.password)?;
                    let sealed_mnemonic_out = self.seal_secret_for_transport(&mnemonic, &request.password)?;
                    created.push(WalletInfo {
                        id,
                        group_id: request.group_id,
                        name,
                        address,
                        chain_type: request.chain_type.clone(),
                        has_private_key: true,
                        has_mnemonic: true,
                        sealed_private_key: Some(sealed_private_key_out),
                        sealed_mnemonic: Some(sealed_mnemonic_out),
                        mnemonic_index: Some(index as i64),
                        remark: request.remark.clone(),
                    });
                }
            }
        }

        tx.commit().await?;
        Ok(created)
    }

    pub async fn get_wallets(&self, group_id: Option<i64>, chain_type: Option<String>, _password: Option<String>) -> Result<Vec<WalletInfo>> {
        let req_chain_type = chain_type
            .clone()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty());

        let wallets = if let Some(gid) = group_id {
            let group = sqlx::query_as::<_, WalletGroup>("SELECT * FROM wallet_groups WHERE id = ?")
                .bind(gid)
                .fetch_optional(&self.pool)
                .await?;

            let group = match group {
                Some(g) => g,
                None => return Ok(Vec::new()),
            };

            let group_chain_type = group
                .chain_type
                .clone()
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty());

            // If request has chain_type, validate it matches group's chain_type
            if let Some(req_ct) = req_chain_type {
                if let Some(group_ct) = group_chain_type {
                    if group_ct != req_ct {
                        return Ok(Vec::new());
                    }
                }
                // Query by group_id and chain_type
                sqlx::query_as::<_, Wallet>("SELECT * FROM wallets WHERE group_id = ? AND lower(trim(chain_type)) = ?")
                    .bind(gid)
                    .bind(&req_ct)
                    .fetch_all(&self.pool)
                    .await?
            } else if group_chain_type.is_some() {
                // Group has chain_type but request doesn't, use group's chain_type
                sqlx::query_as::<_, Wallet>("SELECT * FROM wallets WHERE group_id = ? AND lower(trim(chain_type)) = ?")
                    .bind(gid)
                    .bind(group_chain_type.unwrap())
                    .fetch_all(&self.pool)
                    .await?
            } else {
                // Group doesn't have chain_type, query all wallets in this group
                sqlx::query_as::<_, Wallet>("SELECT * FROM wallets WHERE group_id = ?")
                    .bind(gid)
                    .fetch_all(&self.pool)
                    .await?
            }
        } else if let Some(ct) = req_chain_type {
            // Query by chain_type only (system groups)
            sqlx::query_as::<_, Wallet>("SELECT * FROM wallets WHERE lower(trim(chain_type)) = ?")
                .bind(ct)
                .fetch_all(&self.pool)
                .await?
        } else {
            // No filter, return all wallets
            sqlx::query_as::<_, Wallet>("SELECT * FROM wallets")
                .fetch_all(&self.pool)
                .await?
        };

        Ok(wallets
            .into_iter()
            .map(|w| WalletInfo {
                id: w.id,
                group_id: w.group_id,
                name: w.name,
                address: w.address,
                chain_type: w.chain_type,
                has_private_key: w.encrypted_private_key.is_some(),
                has_mnemonic: w.encrypted_mnemonic.is_some(),
                sealed_private_key: None,
                sealed_mnemonic: None,
                mnemonic_index: w.mnemonic_index,
                remark: w.remark,
            })
            .collect())
    }

    pub async fn delete_wallet(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM wallets WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_wallet(&self, request: UpdateWalletRequest) -> Result<()> {
        sqlx::query(
            "UPDATE wallets SET group_id = ?, name = ?, updated_at = ? WHERE id = ?"
        )
        .bind(request.group_id)
        .bind(request.name)
        .bind(Utc::now())
        .bind(request.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_group(&self, request: UpdateGroupRequest) -> Result<()> {
        sqlx::query("UPDATE wallet_groups SET name = ?, updated_at = ? WHERE id = ?")
            .bind(request.name)
            .bind(Utc::now())
            .bind(request.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_group(&self, id: i64) -> Result<()> {
        // Get all descendant group IDs (including current group) using recursive CTE
        let descendant_ids: Vec<i64> = sqlx::query_scalar::<_, i64>(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id FROM wallet_groups WHERE id = ?
                UNION ALL
                SELECT wg.id FROM wallet_groups wg
                INNER JOIN descendants d ON wg.parent_id = d.id
            )
            SELECT id FROM descendants
            "#
        ).bind(id).fetch_all(&self.pool).await?;

        // Delete all wallets in these groups first
        if !descendant_ids.is_empty() {
            let placeholders = descendant_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!("DELETE FROM wallets WHERE group_id IN ({})", placeholders);
            let mut q = sqlx::query(&query);
            for id in &descendant_ids {
                q = q.bind(id);
            }
            q.execute(&self.pool).await?;
        }

        // Delete the groups (child groups will be deleted by CASCADE)
        sqlx::query("DELETE FROM wallet_groups WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- Helper Methods ---

    fn derive_key(&self, password: &str, salt: &[u8]) -> [u8; 32] {
        let mut key = [0u8; 32];
        let _ = pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, 100_000, &mut key);
        key
    }

    fn seal_secret_for_transport(&self, plaintext: &str, password: &str) -> Result<String> {
        let mut salt = [0u8; 16];
        rand::thread_rng().fill(&mut salt);
        let key = self.derive_key(password, &salt);
        let (cipher_b64, iv) = self.encrypt_data(plaintext.as_bytes(), &key)?;
        Ok(format!(
            "p1:{}:{}:{}",
            hex::encode(salt),
            hex::encode(iv),
            cipher_b64
        ))
    }

    fn open_sealed_secret(&self, sealed: &str, password: &str) -> Result<String> {
        let s = sealed.trim();
        if !s.starts_with("p1:") {
            return Err(anyhow::anyhow!("敏感数据必须使用加密格式传输"));
        }
        let parts: Vec<&str> = s[3..].split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!("密文格式错误"));
        }
        let salt = hex::decode(parts[0])?;
        let iv = hex::decode(parts[1])?;
        let cipher_b64 = parts[2];
        let key = self.derive_key(password, &salt);
        let bytes = self.decrypt_data(cipher_b64, &key, &iv)?;
        Ok(String::from_utf8(bytes)?)
    }

    fn looks_like_db_encrypted_field(&self, field_value: &str) -> bool {
        let parts: Vec<&str> = field_value.split(':').collect();
        if parts.len() != 2 {
            return false;
        }
        let iv_ok = hex::decode(parts[0]).ok().map(|v| v.len() == 16).unwrap_or(false);
        if !iv_ok {
            return false;
        }
        BASE64.decode(parts[1]).is_ok()
    }

    async fn migrate_plaintext_wallet_secrets(&self, mdk: &[u8; 32]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query("SELECT id, encrypted_private_key, encrypted_mnemonic FROM wallets")
            .fetch_all(&mut *tx)
            .await?;

        for row in rows {
            let id: i64 = row.get("id");
            let pk: Option<String> = row.get("encrypted_private_key");
            let mn: Option<String> = row.get("encrypted_mnemonic");

            let mut new_pk: Option<String> = None;
            if let Some(v) = pk.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if !self.looks_like_db_encrypted_field(v) && !v.starts_with("p1:") {
                    let (cipher, iv) = self.encrypt_data(v.as_bytes(), mdk)?;
                    new_pk = Some(format!("{}:{}", hex::encode(iv), cipher));
                }
            }

            let mut new_mn: Option<String> = None;
            if let Some(v) = mn.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if !self.looks_like_db_encrypted_field(v) && !v.starts_with("p1:") {
                    let (cipher, iv) = self.encrypt_data(v.as_bytes(), mdk)?;
                    new_mn = Some(format!("{}:{}", hex::encode(iv), cipher));
                }
            }

            if new_pk.is_some() || new_mn.is_some() {
                sqlx::query(
                    "UPDATE wallets SET encrypted_private_key = COALESCE(?, encrypted_private_key), encrypted_mnemonic = COALESCE(?, encrypted_mnemonic), updated_at = ? WHERE id = ?"
                )
                .bind(new_pk)
                .bind(new_mn)
                .bind(Utc::now())
                .bind(id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    fn resolve_wallet_name(&self, name: Option<&String>, count: u32, index: u32) -> Option<String> {
        let prefix = name.map(|s| s.trim()).filter(|s| !s.is_empty())?;
        if count <= 1 {
            return Some(prefix.to_string());
        }
        Some(format!("{}-{}", prefix, index))
    }

    fn derive_address_from_private_key(&self, chain_type: &str, private_key: &str) -> Result<(String, String)> {
        match chain_type {
            "evm" => {
                let normalized = if private_key.starts_with("0x") || private_key.starts_with("0X") {
                    private_key[2..].to_string()
                } else {
                    private_key.to_string()
                };
                let normalized = normalized.trim().to_string();
                let signer: PrivateKeySigner = normalized.parse().map_err(|e| anyhow::anyhow!("私钥格式错误: {e}"))?;
                Ok((signer.address().to_string(), normalized))
            }
            "solana" => {
                let normalized = private_key.trim().to_string();
                let bytes = bs58::decode(&normalized).into_vec().map_err(|e| anyhow::anyhow!("Solana私钥格式错误: {e}"))?;
                let keypair = Keypair::try_from(bytes.as_slice()).map_err(|e| anyhow::anyhow!("Solana私钥格式错误: {e}"))?;
                Ok((keypair.pubkey().to_string(), normalized))
            }
            _ => Err(anyhow::anyhow!("不支持的链类型")),
        }
    }

    fn derive_from_mnemonic(&self, chain_type: &str, mnemonic: &str, index: u32) -> Result<(String, String)> {
        match chain_type {
            "evm" => {
                let signer = MnemonicBuilder::<English>::default()
                    .phrase(mnemonic)
                    .index(index)?
                    .build()?;
                let pk_bytes = signer.to_bytes();
                let private_key = hex::encode(pk_bytes.as_slice());
                Ok((signer.address().to_string(), private_key))
            }
            "solana" => {
                let seed = self.mnemonic_seed(mnemonic)?;
                let derived = self.derive_slip10_ed25519(&seed, &[44, 501, index, 0])?;
                let keypair = Keypair::new_from_array(derived);
                let private_key = bs58::encode(keypair.to_bytes()).into_string();
                Ok((keypair.pubkey().to_string(), private_key))
            }
            _ => Err(anyhow::anyhow!("不支持的链类型")),
        }
    }

    fn generate_mnemonic(&self, word_count: u32) -> Result<(String, u32)> {
        let count = match word_count {
            12 => 12usize,
            24 => 24usize,
            _ => return Err(anyhow::anyhow!("仅支持12或24词助记词")),
        };
        let mut rng = rand::thread_rng();
        let mnemonic = Mnemonic::<English>::new_with_count(&mut rng, count)?;
        Ok((mnemonic.to_phrase(), word_count))
    }

    fn mnemonic_seed(&self, phrase: &str) -> Result<[u8; 64]> {
        let mnemonic = Mnemonic::<English>::new_from_phrase(phrase)?;
        Ok(mnemonic.to_seed(Some(""))?)
    }

    fn derive_slip10_ed25519(&self, seed: &[u8; 64], path: &[u32]) -> Result<[u8; 32]> {
        let mut mac = HmacSha512::new_from_slice(b"ed25519 seed")?;
        mac.update(seed);
        let mut i = mac.finalize().into_bytes();
        let mut key = [0u8; 32];
        let mut chain_code = [0u8; 32];
        key.copy_from_slice(&i[..32]);
        chain_code.copy_from_slice(&i[32..]);

        for idx in path {
            let hardened = idx | 0x8000_0000;
            let mut mac = HmacSha512::new_from_slice(&chain_code)?;
            mac.update(&[0u8]);
            mac.update(&key);
            mac.update(&hardened.to_be_bytes());
            i = mac.finalize().into_bytes();
            key.copy_from_slice(&i[..32]);
            chain_code.copy_from_slice(&i[32..]);
        }

        Ok(key)
    }

    async fn insert_wallet_row(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        mdk: &[u8; 32],
        group_id: Option<i64>,
        name: Option<String>,
        address: &str,
        chain_type: &str,
        private_key: Option<&str>,
        mnemonic: Option<&str>,
        mnemonic_index: Option<i64>,
        remark: Option<String>,
    ) -> Result<i64> {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wallets WHERE chain_type = ? AND address = ?")
            .bind(chain_type)
            .bind(address)
            .fetch_one(&mut **tx)
            .await?;
        if exists > 0 {
            return Err(anyhow::anyhow!("该地址已存在"));
        }

        let encrypted_pk = if let Some(pk) = private_key {
            let (cipher, iv) = self.encrypt_data(pk.as_bytes(), mdk)?;
            Some(format!("{}:{}", hex::encode(iv), cipher))
        } else {
            None
        };

        let encrypted_mnemonic = if let Some(mn) = mnemonic {
            let (cipher, iv) = self.encrypt_data(mn.as_bytes(), mdk)?;
            Some(format!("{}:{}", hex::encode(iv), cipher))
        } else {
            None
        };

        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO wallets (
                group_id, name, address, chain_type, encrypted_private_key, encrypted_mnemonic, mnemonic_index, remark, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id
            "#
        )
        .bind(group_id)
        .bind(name)
        .bind(address)
        .bind(chain_type)
        .bind(encrypted_pk)
        .bind(encrypted_mnemonic)
        .bind(mnemonic_index)
        .bind(remark)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&mut **tx)
        .await?;

        Ok(id)
    }

    fn encrypt_data(&self, data: &[u8], key: &[u8; 32]) -> Result<(String, Vec<u8>)> {
        let mut iv = [0u8; 16];
        rand::thread_rng().fill(&mut iv);

        let len = data.len();
        let mut buffer = vec![0u8; len + 16];
        buffer[..len].copy_from_slice(data);

        let encryptor = Aes256CbcEnc::new(key.into(), &iv.into());
        let ciphertext_len = encryptor.encrypt_padded_mut::<Pkcs7>(&mut buffer, len)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?
            .len();
        
        buffer.truncate(ciphertext_len);
        Ok((BASE64.encode(&buffer), iv.to_vec()))
    }

    fn decrypt_data(&self, ciphertext_base64: &str, key: &[u8; 32], iv: &[u8]) -> Result<Vec<u8>> {
        let mut ciphertext = BASE64.decode(ciphertext_base64)?;
        let decryptor = Aes256CbcDec::new(key.into(), iv.into());
        
        let plaintext = decryptor.decrypt_padded_mut::<Pkcs7>(&mut ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
        
        Ok(plaintext.to_vec())
    }

    fn decrypt_field(&self, field_value: &str, key: &[u8; 32]) -> Result<String> {
        let parts: Vec<&str> = field_value.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Format error"));
        }
        let iv = hex::decode(parts[0])?;
        let ciphertext = parts[1];
        let bytes = self.decrypt_data(ciphertext, key, &iv)?;
        Ok(String::from_utf8(bytes)?)
    }

    async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO app_config (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = ?"
        )
        .bind(key)
        .bind(value)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_config(&self, key: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT value FROM app_config WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(r) = row {
            Ok(Some(r.get("value")))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod wallet_manager_get_wallets_tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn get_wallets_filters_by_chain_type() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let service = WalletManagerService::new(pool.clone());
        service.init_tables().await.unwrap();

        sqlx::query("INSERT INTO wallet_groups (id, parent_id, name, chain_type, created_at, updated_at) VALUES (7, NULL, 'g7', 'evm', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wallet_groups (id, parent_id, name, chain_type, created_at, updated_at) VALUES (8, NULL, 'g8', 'solana', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO wallets (group_id, name, address, chain_type, created_at, updated_at) VALUES (7, 'a', '0x1', 'evm', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wallets (group_id, name, address, chain_type, created_at, updated_at) VALUES (8, 'b', 'S1', 'solana', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wallets (group_id, name, address, chain_type, created_at, updated_at) VALUES (8, 'c', 'S2', ' SOLANA ', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await
            .unwrap();

        let all_solana = service.get_wallets(None, Some("solana".into()), None).await.unwrap();
        assert!(all_solana.iter().all(|w| w.chain_type.trim().eq_ignore_ascii_case("solana")));

        let g8_solana = service.get_wallets(Some(8), Some("solana".into()), None).await.unwrap();
        assert!(g8_solana.iter().all(|w| w.group_id == Some(8)));
        assert!(g8_solana.iter().all(|w| w.chain_type.trim().eq_ignore_ascii_case("solana")));

        let g8_evm = service.get_wallets(Some(8), Some("evm".into()), None).await.unwrap();
        assert!(g8_evm.is_empty());
    }
}

#[cfg(test)]
mod wallet_manager_secrets_tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn seal_and_open_roundtrip() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let service = WalletManagerService::new(pool);
        let sealed = service.seal_secret_for_transport("hello", "pass").unwrap();
        assert!(sealed.starts_with("p1:"));
        let opened = service.open_sealed_secret(&sealed, "pass").unwrap();
        assert_eq!(opened, "hello");
        assert!(service.open_sealed_secret("hello", "pass").is_err());
    }

    #[tokio::test]
    async fn migrate_plaintext_wallet_secrets_encrypts() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let service = WalletManagerService::new(pool.clone());
        service.init_tables().await.unwrap();
        service.init_password("pass").await.unwrap();

        sqlx::query("INSERT INTO wallets (group_id, name, address, chain_type, encrypted_private_key, encrypted_mnemonic, created_at, updated_at) VALUES (NULL, 'n', '0x1', 'evm', 'plain_private_key', 'plain mnemonic', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await
            .unwrap();

        assert!(service.unlock("pass").await.unwrap());

        let row = sqlx::query("SELECT encrypted_private_key, encrypted_mnemonic FROM wallets WHERE address = '0x1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        let enc_pk: String = row.get("encrypted_private_key");
        let enc_mn: String = row.get("encrypted_mnemonic");
        assert!(service.looks_like_db_encrypted_field(&enc_pk));
        assert!(service.looks_like_db_encrypted_field(&enc_mn));
        assert_ne!(enc_pk, "plain_private_key");
        assert_ne!(enc_mn, "plain mnemonic");
    }

    #[tokio::test]
    async fn create_wallets_returns_sealed_secrets() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let service = WalletManagerService::new(pool.clone());
        service.init_tables().await.unwrap();
        service.init_password("pass").await.unwrap();

        let pk = format!("0x{}", "1".repeat(64));
        let sealed_pk = service.seal_secret_for_transport(&pk, "pass").unwrap();

        let created = service
            .create_wallets(
                CreateWalletsRequest {
                    group_id: None,
                    name: Some("w".into()),
                    chain_type: "evm".into(),
                    mode: CreateWalletsMode::PrivateKeyImport,
                    sealed_mnemonic: None,
                    sealed_private_key: Some(sealed_pk),
                    count: 1,
                    start_index: None,
                    word_count: None,
                    remark: None,
                    password: "pass".into(),
                },
                None,
            )
            .await
            .unwrap();

        assert_eq!(created.len(), 1);
        let w = &created[0];
        assert!(w.has_private_key);
        assert!(!w.has_mnemonic);
        let sealed_out = w.sealed_private_key.clone().unwrap();
        assert_ne!(sealed_out, pk.trim_start_matches("0x"));
        let opened = service.open_sealed_secret(&sealed_out, "pass").unwrap();
        assert_eq!(opened, pk.trim_start_matches("0x"));
    }
}
