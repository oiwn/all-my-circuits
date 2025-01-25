use anyhow::{Context, Result};
use ignore::WalkBuilder;
use log::{debug, info};
use std::path::{Path, PathBuf};

const EXCLUDED_FILES: &[&str] = &[".amc.toml"];

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

        info!("Starting file walk in directory: {}", base_path.display());
        info!("Looking for files with extensions: {:?}", self.extensions);

        // Create the builder
        let mut builder = WalkBuilder::new(&base_path);
        builder
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .require_git(false)
            .ignore(true);

        // Add the gitignore file if it exists
        let gitignore_path = base_path.join(".gitignore");
        if gitignore_path.exists() {
            info!("Found .gitignore at: {}", gitignore_path.display());
            if let Some(err) = builder.add_ignore(&gitignore_path) {
                eprintln!("Warning: Failed to add .gitignore file: {}", err);
            }
        }

        // Build the walker and collect files
        let files: Vec<FileEntry> = builder
            .build()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
            .filter(|entry| {
                let is_valid = self.is_valid_extension(entry.path());
                debug!(
                    "Checking file: {} - {}",
                    entry.path().display(),
                    if is_valid { "included" } else { "skipped" }
                );
                is_valid
            })
            .map(|entry| {
                let absolute_path = entry.path().to_path_buf();
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
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if EXCLUDED_FILES.contains(&file_name) {
                return false;
            }
        }
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.extensions.iter().any(|allowed_ext| allowed_ext == ext))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_directory() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;

        // First create .gitignore file
        let gitignore_content = "*.txt\ntarget/\n.git/\n";
        fs::write(temp_dir.path().join(".gitignore"), gitignore_content)?;

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
            fs::write(full_path, content)?;
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
    fn test_gitignore_respecting() -> Result<()> {
        let temp_dir = setup_test_directory()?;

        // Create a file in target directory
        fs::create_dir_all(temp_dir.path().join("target"))?;
        fs::write(temp_dir.path().join("target/ignored.rs"), "ignored content")?;

        let walker = FileWalker::new(vec!["rs".to_string(), "txt".to_string()]);
        let files = walker.walk(temp_dir.path())?;

        // Print debug information
        println!("Files found:");
        for file in &files {
            println!("  {:?}", file.relative_path);
        }

        // Should not find .txt files or files in target/
        for file in &files {
            let path_str = file.relative_path.to_string_lossy();
            assert!(
                !path_str.contains("target/"),
                "Found file in target/: {}",
                path_str
            );
            assert!(!path_str.ends_with(".txt"), "Found .txt file: {}", path_str);
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

    #[test]
    fn test_exclude_config_file() -> Result<()> {
        let temp_dir = setup_test_directory()?;
        fs::write(temp_dir.path().join(".amc.toml"), "content")?;

        let walker = FileWalker::new(vec!["toml".to_string()]);
        let files = walker.walk(temp_dir.path())?;

        for file in &files {
            assert_ne!(
                file.relative_path.file_name().unwrap().to_str().unwrap(),
                ".amc.toml"
            );
        }

        Ok(())
    }
}
