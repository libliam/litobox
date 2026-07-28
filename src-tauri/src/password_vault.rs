use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use base64::engine::{general_purpose, Engine as _};
use uuid::Uuid;
use rusqlite::{params, OptionalExtension};

use crate::db::with_conn;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Credential {
    pub id: Option<i64>,
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultCredential {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

fn derive_key(master_password: &str, salt: &str) -> Vec<u8> {
    let mut key = master_password.as_bytes().to_vec();
    key.extend_from_slice(salt.as_bytes());
    
    for _ in 0..10000 {
        let mut hasher = Sha256::new();
        hasher.update(&key);
        key = hasher.finalize().to_vec();
    }
    
    key
}

fn encrypt(password: &str, key: &[u8]) -> String {
    let bytes = password.as_bytes();
    let mut encrypted = Vec::with_capacity(bytes.len());
    
    for (i, &byte) in bytes.iter().enumerate() {
        encrypted.push(byte ^ key[i % key.len()]);
    }
    
    general_purpose::STANDARD.encode(&encrypted)
}

fn decrypt(encrypted: &str, key: &[u8]) -> Result<String, String> {
    let bytes = general_purpose::STANDARD.decode(encrypted).map_err(|e| e.to_string())?;
    let mut decrypted = Vec::with_capacity(bytes.len());
    
    for (i, &byte) in bytes.iter().enumerate() {
        decrypted.push(byte ^ key[i % key.len()]);
    }
    
    String::from_utf8(decrypted).map_err(|e| e.to_string())
}

fn generate_salt() -> String {
    Uuid::new_v4().to_string()
}

#[tauri::command]
pub fn pv_has_master_password() -> Result<bool, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_master_hash'")
            .map_err(|e| e.to_string())?;
        let result: Option<String> = stmt.query_row([], |row| Ok(row.get::<_, String>(0)?))
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result.is_some())
    })
}

#[tauri::command]
pub fn pv_set_master_password(password: String) -> Result<(), String> {
    if password.len() < 4 {
        return Err("密码长度至少4位".to_string());
    }
    
    let salt = generate_salt();
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let hash = general_purpose::STANDARD.encode(&hasher.finalize());
    
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO config (key, value) VALUES ('password_vault_master_hash', ?1)
             ON CONFLICT(key) DO UPDATE SET value = ?1",
            params![hash],
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO config (key, value) VALUES ('password_vault_salt', ?1)
             ON CONFLICT(key) DO UPDATE SET value = ?1",
            params![salt],
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO config (key, value) VALUES ('password_vault_master_plain', ?1)
             ON CONFLICT(key) DO UPDATE SET value = ?1",
            params![password],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn pv_verify_master_password(password: String) -> Result<bool, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_master_hash'")
            .map_err(|e| e.to_string())?;
        let hash: Option<String> = stmt.query_row([], |row| Ok(row.get::<_, String>(0)?))
            .optional()
            .map_err(|e| e.to_string())?;
        
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        let salt: Option<String> = stmt.query_row([], |row| Ok(row.get::<_, String>(0)?))
            .optional()
            .map_err(|e| e.to_string())?;
        
        match (hash, salt) {
            (Some(h), Some(s)) => {
                let mut hasher = Sha256::new();
                hasher.update(password.as_bytes());
                hasher.update(s.as_bytes());
                let computed = general_purpose::STANDARD.encode(&hasher.finalize());
                Ok(computed == h)
            }
            _ => Ok(false),
        }
    })
}

#[tauri::command]
pub fn pv_list_credentials(master_password: String) -> Result<Vec<VaultCredential>, String> {
    let salt: Option<String> = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| Ok(row.get::<_, String>(0)?))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, url, username, encrypted_password, notes, created_at, updated_at, salt
                      FROM password_vault ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        
        let mut credentials = Vec::new();
        for row in rows {
            let (id, name, url, username, encrypted_password, notes, created_at, updated_at, credential_salt) = match row {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("pv_list_credentials: 读取凭据行失败: {}", e);
                    continue;
                }
            };
            
            let mut hasher = Sha256::new();
            hasher.update(&key);
            hasher.update(credential_salt.as_bytes());
            let credential_key = hasher.finalize().to_vec();
            
            let password = match decrypt(&encrypted_password, &credential_key) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("pv_list_credentials: 解密凭据失败 (id={}): {}", id, e);
                    continue;
                }
            };
            
            credentials.push(VaultCredential {
                id,
                name,
                url,
                username,
                password,
                notes,
                created_at,
                updated_at,
            });
        }
        
        Ok(credentials)
    })
}

#[tauri::command]
pub fn pv_search_credentials(master_password: String, query: String) -> Result<Vec<VaultCredential>, String> {
    let salt: Option<String> = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| Ok(row.get::<_, String>(0)?))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    with_conn(|conn| {
        let like = format!("%{}%", query);
        let mut stmt = conn
            .prepare("SELECT id, name, url, username, encrypted_password, notes, created_at, updated_at, salt
                      FROM password_vault
                      WHERE name LIKE ?1 OR username LIKE ?1 OR url LIKE ?1
                      ORDER BY updated_at DESC")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![like], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        
        let mut credentials = Vec::new();
        for row in rows {
            let (id, name, url, username, encrypted_password, notes, created_at, updated_at, credential_salt) = match row {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("pv_search_credentials: 读取凭据行失败: {}", e);
                    continue;
                }
            };
            
            let mut hasher = Sha256::new();
            hasher.update(&key);
            hasher.update(credential_salt.as_bytes());
            let credential_key = hasher.finalize().to_vec();
            
            let password = match decrypt(&encrypted_password, &credential_key) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("pv_search_credentials: 解密凭据失败 (id={}): {}", id, e);
                    continue;
                }
            };
            
            credentials.push(VaultCredential {
                id,
                name,
                url,
                username,
                password,
                notes,
                created_at,
                updated_at,
            });
        }
        
        Ok(credentials)
    })
}

#[tauri::command]
pub fn pv_add_credential(master_password: String, credential: Credential) -> Result<i64, String> {
    let salt: Option<String> = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| Ok(row.get::<_, String>(0)?))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    let credential_salt = generate_salt();
    let mut hasher = Sha256::new();
    hasher.update(&key);
    hasher.update(credential_salt.as_bytes());
    let credential_key = hasher.finalize().to_vec();
    
    let encrypted_password = encrypt(&credential.password, &credential_key);
    
    with_conn(|conn| {
        conn.execute(
            "INSERT INTO password_vault (name, url, username, encrypted_password, notes, salt)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![credential.name, credential.url, credential.username, encrypted_password, credential.notes, credential_salt],
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    })
}

#[tauri::command]
pub fn pv_update_credential(master_password: String, credential: Credential) -> Result<(), String> {
    let id = credential.id.ok_or("凭据ID不能为空".to_string())?;
    
    let salt: Option<String> = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| Ok(row.get::<_, String>(0)?))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    let credential_salt = generate_salt();
    let mut hasher = Sha256::new();
    hasher.update(&key);
    hasher.update(credential_salt.as_bytes());
    let credential_key = hasher.finalize().to_vec();
    
    let encrypted_password = encrypt(&credential.password, &credential_key);
    
    with_conn(|conn| {
        conn.execute(
            "UPDATE password_vault SET name = ?1, url = ?2, username = ?3, 
             encrypted_password = ?4, notes = ?5, salt = ?6, updated_at = datetime('now')
             WHERE id = ?7",
            params![credential.name, credential.url, credential.username, encrypted_password, credential.notes, credential_salt, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn pv_delete_credential(id: i64) -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM password_vault WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn pv_reset_master_password() -> Result<(), String> {
    with_conn(|conn| {
        conn.execute("DELETE FROM password_vault", [])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM config WHERE key LIKE 'password_vault_%'", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[tauri::command]
pub fn pv_import_credentials(master_password: String, credentials: Vec<Credential>) -> Result<usize, String> {
    let salt: Option<String> = with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?;
        stmt.query_row([], |row| Ok(row.get::<_, String>(0)?))
            .optional()
            .map_err(|e| e.to_string())
    })?;
    
    let salt = salt.ok_or("主密码盐不存在".to_string())?;
    let key = derive_key(&master_password, &salt);
    
    let mut count = 0;
    with_conn(|conn| {
        for cred in &credentials {
            if cred.name.is_empty() || cred.password.is_empty() {
                continue;
            }
            let credential_salt = generate_salt();
            let mut hasher = Sha256::new();
            hasher.update(&key);
            hasher.update(credential_salt.as_bytes());
            let credential_key = hasher.finalize().to_vec();
            let encrypted_password = encrypt(&cred.password, &credential_key);
            
            conn.execute(
                "INSERT INTO password_vault (name, url, username, encrypted_password, notes, salt)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![cred.name, cred.url, cred.username, encrypted_password, cred.notes, credential_salt],
            ).map_err(|e| e.to_string())?;
            count += 1;
        }
        Ok(())
    })?;
    
    Ok(count)
}

#[tauri::command]
pub fn pv_change_master_password(old_password: String, new_password: String) -> Result<(), String> {
    if new_password.len() < 4 {
        return Err("新密码长度至少4位".to_string());
    }

    with_conn(|conn| {
        // 1. 验证旧密码
        let hash: String = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_master_hash'")
            .map_err(|e| e.to_string())?
            .query_row([], |row| row.get::<_, String>(0))
            .map_err(|_| "主密码验证失败".to_string())?;

        let salt: String = conn
            .prepare("SELECT value FROM config WHERE key = 'password_vault_salt'")
            .map_err(|e| e.to_string())?
            .query_row([], |row| row.get::<_, String>(0))
            .map_err(|_| "主密码盐不存在".to_string())?;

        {
            let mut hasher = Sha256::new();
            hasher.update(old_password.as_bytes());
            hasher.update(salt.as_bytes());
            let computed = general_purpose::STANDARD.encode(&hasher.finalize());
            if computed != hash {
                return Err("旧密码错误".to_string());
            }
        }

        // 2. 读取所有凭据，用旧密钥解密
        let old_key = derive_key(&old_password, &salt);

        let mut stmt = conn
            .prepare("SELECT id, encrypted_password, salt FROM password_vault")
            .map_err(|e| e.to_string())?;
        let credential_rows: Vec<(i64, String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut decrypted_credentials: Vec<(i64, String)> = Vec::new();
        for (id, enc_pwd, cred_salt) in &credential_rows {
            let mut hasher = Sha256::new();
            hasher.update(&old_key);
            hasher.update(cred_salt.as_bytes());
            let cred_key = hasher.finalize().to_vec();
            match decrypt(enc_pwd, &cred_key) {
                Ok(p) => decrypted_credentials.push((*id, p)),
                Err(e) => eprintln!("pv_change_master_password: 跳过解密失败的凭据 (id={}): {}", id, e),
            }
        }

        // 3. 更新主密码配置
        let new_salt = generate_salt();
        {
            let mut hasher = Sha256::new();
            hasher.update(new_password.as_bytes());
            hasher.update(new_salt.as_bytes());
            let new_hash = general_purpose::STANDARD.encode(&hasher.finalize());

            conn.execute(
                "UPDATE config SET value = ?1 WHERE key = 'password_vault_master_hash'",
                params![new_hash],
            ).map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE config SET value = ?1 WHERE key = 'password_vault_salt'",
                params![new_salt],
            ).map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE config SET value = ?1 WHERE key = 'password_vault_master_plain'",
                params![new_password],
            ).map_err(|e| e.to_string())?;
        }

        // 4. 用新密钥重新加密凭据
        let new_key = derive_key(&new_password, &new_salt);
        for (id, password) in &decrypted_credentials {
            let cred_salt = generate_salt();
            let mut hasher = Sha256::new();
            hasher.update(&new_key);
            hasher.update(cred_salt.as_bytes());
            let cred_key = hasher.finalize().to_vec();
            let encrypted = encrypt(password, &cred_key);

            conn.execute(
                "UPDATE password_vault SET encrypted_password = ?1, salt = ?2, updated_at = datetime('now') WHERE id = ?3",
                params![encrypted, cred_salt, id],
            ).map_err(|e| e.to_string())?;
        }

        Ok(())
    })
}

/// 导入时检测重复：基于 (name + username) 匹配已有数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImportCredential {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DuplicateResult {
    pub index: usize,
    pub name: String,
    pub url: String,
    pub username: String,
}

#[tauri::command]
pub fn pv_check_duplicates(credentials: Vec<ImportCredential>) -> Result<Vec<DuplicateResult>, String> {
    with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT name, url, username FROM password_vault")
            .map_err(|e| e.to_string())?;
        let existing: Vec<(String, String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let mut duplicates = Vec::new();
        for (i, cred) in credentials.iter().enumerate() {
            for (ename, _eurl, euser) in &existing {
                if &cred.name == ename && &cred.username == euser {
                    duplicates.push(DuplicateResult {
                        index: i,
                        name: cred.name.clone(),
                        url: cred.url.clone(),
                        username: cred.username.clone(),
                    });
                    break;
                }
            }
        }
        Ok(duplicates)
    })
}

#[tauri::command]
pub fn pv_batch_delete(ids: Vec<i64>) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    with_conn(|conn| {
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!("DELETE FROM password_vault WHERE id IN ({})", placeholders.join(","));
        let params: Vec<&dyn rusqlite::types::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();
        let count = conn.execute(&sql, params.as_slice()).map_err(|e| e.to_string())?;
        Ok(count)
    })
}
