//! On-disk token cache. It holds live credentials: keep it private.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AuthError, ChatKeys};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCache {
    pub msa: Option<CachedMsa>,
    pub minecraft: Option<CachedMinecraft>,
    pub chat_keys: Option<ChatKeys>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMsa {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix seconds.
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMinecraft {
    pub access_token: String,
    /// Unix seconds.
    pub expires_at: i64,
    pub name: String,
    pub id: Uuid,
}

impl TokenCache {
    pub fn load(path: &Path) -> Result<Self, AuthError> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(serde_json::from_str(&s)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), AuthError> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        let contents = serde_json::to_string_pretty(self)?;
        let mut options = OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&tmp)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(contents.as_bytes())?;
        std::fs::rename(tmp, path)?;
        Ok(())
    }
}

#[cfg(unix)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn save_restricts_cache_permissions_before_reusing_temp_file() {
        let dir = std::env::temp_dir().join(format!("rapidbot-cache-{}", Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("tokens.json");
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, b"stale contents").unwrap();
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o644)).unwrap();

        TokenCache::default().save(&path).unwrap();

        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
