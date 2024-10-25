use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FileWalker {
    extensions: Vec<String>,
}

#[derive(Debug)]
pub struct FileEntry {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
}

impl FileWalker {
    pub fn new(extensions: Vec<String>) -> Self {
        Self {
            extensions: extensions
                .into_iter()
                .map(|ext| ext.trim_start_matches('.').to_string())
                .collect(),
        }
    }

    pub fn walk<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<FileEntry>> {
        let base_path = if dir.as_ref() == Path::new(".") {
            std::env::current_dir()?
        } else {
            dir.as_ref()
                .canonicalize()
                .context("Failed to resolve directory path")?
        };

        let files = WalkDir::new(&base_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| self.is_valid_extension(e.path()))
            .map(|e| {
                let absolute_path = e.path().to_path_buf();
                let relative_path = absolute_path
                    .strip_prefix(&base_path)
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|_| absolute_path.clone());
                FileEntry {
                    absolute_path,
                    relative_path,
                }
            })
            .collect();

        Ok(files)
    }

    fn is_valid_extension(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.extensions.iter().any(|allowed_ext| allowed_ext == ext))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_directory() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;

        // Create some test files with different extensions
        let files = vec![
            ("test1.rs", "content1"),
            ("test2.rs", "content2"),
            ("test3.txt", "content3"),
            ("subdir/test4.rs", "content4"),
            ("test5", "content5"), // No extension
        ];

        for (path, content) in files {
            let full_path = temp_dir.path().join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = File::create(full_path)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(temp_dir)
    }

    #[test]
    fn test_walk_with_extensions() -> Result<()> {
        let temp_dir = setup_test_directory()?;
        let walker = FileWalker::new(vec!["rs".to_string()]);

        let files = walker.walk(temp_dir.path())?;

        // Should find exactly 3 .rs files
        assert_eq!(files.len(), 3);

        // Verify all found files have .rs extension
        for file in files {
            assert_eq!(file.absolute_path.extension().unwrap(), "rs");
        }

        Ok(())
    }

    #[test]
    fn test_relative_paths() -> Result<()> {
        let temp_dir = setup_test_directory()?;
        let walker = FileWalker::new(vec!["rs".to_string()]);

        let files = walker.walk(temp_dir.path())?;

        for file in files {
            // Relative paths should not be absolute
            assert!(!file.relative_path.is_absolute());

            // Absolute paths should be absolute
            assert!(file.absolute_path.is_absolute());

            // Joining base path and relative path should give us the absolute path
            assert_eq!(
                temp_dir.path().join(&file.relative_path).canonicalize()?,
                file.absolute_path.canonicalize()?
            );
        }

        Ok(())
    }
}
