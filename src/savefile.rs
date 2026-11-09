// Where the game keeps its files, and writing them safely.
//
// Files live next to the executable (not wherever the game happened to be started from).
// A save is written to a temporary file first and then swapped in, with the previous save
// kept as a backup, so a crash or a full disk can never leave you with half a save.
use std::path::PathBuf;

/// the folder the game's files live in: beside the executable
pub fn data_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_else(|| PathBuf::from("."))
}

/// a file beside the executable; if it isn't there yet but one exists in the current
/// folder (older versions saved there), that one is used for reading
pub fn path(name: &str) -> PathBuf {
    data_dir().join(name)
}
pub fn read(name: &str) -> Option<String> {
    std::fs::read_to_string(path(name)).ok().or_else(|| std::fs::read_to_string(name).ok())
}
pub fn exists(name: &str) -> bool {
    path(name).exists() || std::path::Path::new(name).exists()
}

/// write via a temporary file, keeping the old version as `<name>.bak`
pub fn write_safe(name: &str, text: &str) -> std::io::Result<()> {
    let target = path(name);
    let tmp = path(&format!("{}.tmp", name));
    std::fs::write(&tmp, text)?;
    // make sure what we wrote reads back whole before replacing anything
    if std::fs::read_to_string(&tmp)? != text {
        return Err(std::io::Error::other("the save did not read back correctly"));
    }
    if target.exists() {
        let _ = std::fs::copy(&target, path(&format!("{}.bak", name)));
    }
    std::fs::rename(&tmp, &target)
}

/// a small settings file: failures don't matter much, but never leave it half-written
pub fn write_quiet(name: &str, text: &str) {
    let _ = write_safe(name, text);
}
