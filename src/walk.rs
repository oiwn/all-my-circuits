use anyhow::{Context, Result};
use ignore::WalkBuilder;
use log::{debug, info};
use std::path::{Path, PathBuf};

const EXCLUDED_FILES: &[&str] = &[".amc.toml"];

pub struct FileWalker {
    extensions: Vec<String>,
    excluded_folders: Vec<String>,
}

#[derive(Debug)]
pub struct FileEntry {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
}

impl FileWalker {
    pub fn new(extensions: Vec<String>, excluded_folders: Vec<String>) -> Self {
        Self {
            extensions: extensions
                .into_iter()
                .map(|ext| ext.trim_start_matches('.').to_string())
                .collect(),
            excluded_folders,
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
            .filter(|entry| !self.is_excluded_directory(entry.path())) // Add this line
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

        // Build the walker and collect files
        /* let files: Vec<FileEntry> = builder
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
        .collect(); */

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

    fn is_excluded_directory(&self, path: &Path) -> bool {
        if self.excluded_folders.is_empty() {
            return false;
        }

        // Check if any parent directory component matches excluded folders
        path.ancestors()
            .filter_map(|ancestor| ancestor.file_name()?.to_str())
            .any(|folder_name| {
                self.excluded_folders.contains(&folder_name.to_string())
            })
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
        let walker = FileWalker::new(vec!["rs".to_string()], vec![]);

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

        let walker =
            FileWalker::new(vec!["rs".to_string(), "txt".to_string()], vec![]);
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
        let walker = FileWalker::new(vec!["rs".to_string()], vec![]);

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

        let walker = FileWalker::new(vec!["toml".to_string()], vec![]);
        let files = walker.walk(temp_dir.path())?;

        for file in &files {
            assert_ne!(
                file.relative_path.file_name().unwrap().to_str().unwrap(),
                ".amc.toml"
            );
        }

        Ok(())
    }

    #[test]
    fn test_exclude_folders() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create directory structure
        let files = vec![
            ("src/main.rs", "main content"),
            ("src/lib.rs", "lib content"),
            ("target/debug/app.rs", "debug content"),
            ("target/release/app.rs", "release content"),
            ("docs/target/example.rs", "docs example"), // nested 'target' folder
            ("other/file.rs", "other content"),
        ];

        for (path, content) in files {
            let full_path = temp_dir.path().join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(full_path, content)?;
        }

        // Test excluding 'target' folder
        let walker =
            FileWalker::new(vec!["rs".to_string()], vec!["target".to_string()]);
        let files = walker.walk(temp_dir.path())?;

        println!("Files found:");
        for file in &files {
            println!("  {:?}", file.relative_path);
        }

        // Should find: src/main.rs, src/lib.rs, other/file.rs
        // Should NOT find: target/debug/app.rs, target/release/app.rs
        // Question: Should it find docs/target/example.rs?

        let found_paths: Vec<String> = files
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(found_paths.contains(&"src/main.rs".to_string()));
        assert!(found_paths.contains(&"src/lib.rs".to_string()));
        assert!(found_paths.contains(&"other/file.rs".to_string()));

        // These should be excluded
        assert!(!found_paths.iter().any(|p| p.contains("target/debug")));
        assert!(!found_paths.iter().any(|p| p.contains("target/release")));

        // The key question: should docs/target/example.rs be excluded?
        // Current implementation would exclude it, but maybe we want only top-level exclusion?

        Ok(())
    }
}
