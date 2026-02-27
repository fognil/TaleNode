use std::collections::HashMap;
use std::path::{Path, PathBuf};

enum CacheEntry {
    Loaded(egui::TextureHandle),
    Failed,
}

/// Loads and caches character portrait images as egui textures.
pub struct PortraitCache {
    entries: HashMap<PathBuf, CacheEntry>,
}

impl PortraitCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get or lazily load a portrait texture for the given relative path.
    /// Returns None if path is empty, file missing, or image decode fails.
    pub fn get_or_load(
        &mut self,
        ctx: &egui::Context,
        portrait_path: &str,
        project_dir: Option<&Path>,
    ) -> Option<&egui::TextureHandle> {
        if portrait_path.is_empty() {
            return None;
        }
        let resolved = resolve_path(portrait_path, project_dir);
        if !self.entries.contains_key(&resolved) {
            let entry = load_image(ctx, &resolved);
            self.entries.insert(resolved.clone(), entry);
        }
        match self.entries.get(&resolved) {
            Some(CacheEntry::Loaded(handle)) => Some(handle),
            _ => None,
        }
    }
}

fn resolve_path(portrait_path: &str, project_dir: Option<&Path>) -> PathBuf {
    let path = Path::new(portrait_path);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match project_dir {
        Some(dir) => dir.join(path),
        None => PathBuf::from(portrait_path),
    }
}

/// Make a path relative to the project directory if possible.
pub(crate) fn make_relative_path(path: &Path, project_dir: Option<&Path>) -> String {
    if let Some(dir) = project_dir {
        if let Ok(rel) = path.strip_prefix(dir) {
            return rel.display().to_string();
        }
    }
    path.display().to_string()
}

fn load_image(ctx: &egui::Context, path: &Path) -> CacheEntry {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return CacheEntry::Failed,
    };
    let img = match image::load_from_memory(&bytes) {
        Ok(i) => i.into_rgba8(),
        Err(_) => return CacheEntry::Failed,
    };
    let size = [img.width() as usize, img.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "portrait".to_string());
    let handle = ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR);
    CacheEntry::Loaded(handle)
}
