#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

/// JSON token store backed by a file with owner-only permissions on Unix.
///
/// # Examples
///
/// ```no_run
/// use schwab::auth::FileTokenStore;
///
/// let store = FileTokenStore::new("~/.config/schwab/token.json");
/// assert_eq!(store.path().to_str().unwrap(), "~/.config/schwab/token.json");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileTokenStore {
    path: PathBuf,
}

impl FileTokenStore {
    /// Creates a file token store at the provided path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the token file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl TokenStore for FileTokenStore {
    fn save(&self, token_file: &TokenFile) -> Result<()> {
        if let Some(parent) = real_parent(&self.path) {
            fs::create_dir_all(parent).map_err(Error::Io)?;
            set_private_dir_permissions(parent)?;
        }

        let temp_path = self.path.with_extension("tmp");
        let _ = fs::remove_file(&temp_path);
        let encoded = serde_json::to_vec_pretty(token_file).map_err(Error::Encode)?;
        let mut temp_file = private_file(&temp_path)?;
        temp_file.write_all(&encoded).map_err(Error::Io)?;
        temp_file.write_all(b"\n").map_err(Error::Io)?;
        temp_file.sync_all().map_err(Error::Io)?;
        drop(temp_file);
        fs::rename(&temp_path, &self.path).map_err(Error::Io)?;
        sync_parent_dir(&self.path)?;
        Ok(())
    }

    fn load(&self) -> Result<TokenFile> {
        let contents = fs::read_to_string(&self.path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::AuthRequired
            } else {
                Error::Io(error)
            }
        })?;
        serde_json::from_str(&contents).map_err(Error::Json)
    }
}

/// In-memory token store useful for tests and short-lived tools.
///
/// # Examples
///
/// ```
/// use schwab::auth::MemoryTokenStore;
///
/// let store = MemoryTokenStore::new();
/// ```
#[derive(Debug, Default)]
pub struct MemoryTokenStore {
    token_file: Mutex<Option<TokenFile>>,
}

impl MemoryTokenStore {
    /// Creates an empty memory token store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl TokenStore for MemoryTokenStore {
    fn save(&self, token_file: &TokenFile) -> Result<()> {
        let mut guard = self
            .token_file
            .lock()
            .map_err(|_| Error::AuthCallback("memory token store lock poisoned".to_string()))?;
        *guard = Some(token_file.clone());
        Ok(())
    }

    fn load(&self) -> Result<TokenFile> {
        self.token_file
            .lock()
            .map_err(|_| Error::AuthCallback("memory token store lock poisoned".to_string()))?
            .clone()
            .ok_or(Error::AuthRequired)
    }
}

fn private_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(Error::Io)
}

fn real_parent(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

fn set_private_dir_permissions(_path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(_path, fs::Permissions::from_mode(0o700)).map_err(Error::Io)?;
    }
    Ok(())
}

fn sync_parent_dir(_path: &Path) -> Result<()> {
    #[cfg(not(windows))]
    {
        if let Some(parent) = real_parent(_path) {
            File::open(parent)
                .and_then(|file| file.sync_all())
                .map_err(Error::Io)?;
        }
    }
    Ok(())
}
