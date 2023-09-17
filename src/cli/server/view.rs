use std::collections::HashMap;
use std::fs::{
    ReadDir,
    DirEntry
};
use std::io::PathBuf;
use std::fs::read_dir;

type ViewFile = &'static str;
type ViewFiles = HashMap<Hierarchy, ViewFile>;
enum Hierarchy {
    FileName(String),
    DirName(ViewFiles)
};

pub fn get_view_files() -> ViewFiles {
    let mut dirs: Vec<PathBuf> = vec!(PathBuf::from("./src/"))
}