#[derive(Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub selected: bool,
}

impl FileNode {
    pub fn new(
        name: &str,
        path: &str,
        is_dir: bool,
        size: u64,
        selected: bool,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir,
            size,
            selected,
        }
    }
}
