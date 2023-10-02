// used modules
use std::env;

// used types
use std::fs::DirEntry;
use std::path::{
    PathBuf,
    Path
};

// used functions
use std::fs::{
    read_dir,
    write
};

fn main() {
    println!("cargo:rerun-if-changed=./react_app/view_files");

    let manifest_dir: String = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut in_dir = PathBuf::from(&manifest_dir);
    in_dir.push("react_app/view_files");
    
    let mut view_rs: String = String::from("
use std::collections::HashMap;

type ContentHeader = String;
type ViewFile = &'static [u8];
type ViewFiles = HashMap<URN, Hierarchy>;

#[derive(Clone, Hash, Eq, PartialEq)]
enum URN {
    FileName(String),
    DirName(String)
}

#[derive(Clone)]
enum Hierarchy {
    File((ViewFile, ContentHeader)),
    Dir(ViewFiles)
}

fn get_view_files() -> ViewFiles {
    HashMap::from([
");
    view_rs.push_str(visit_dirs(in_dir.as_path()).as_str());
    view_rs.push_str("    ])\n}");

    let out_dir: String = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("view.rs");
    write(dest_path, &view_rs).unwrap();
}

fn visit_dirs(dir: &Path) -> String {
    let mut view_rs: String = String::new();
    for entry in read_dir(dir).unwrap() {
        let entry: DirEntry = entry.unwrap();
        let path: PathBuf = entry.path();
        if path.is_file() {
            let push_string = |content_type: &str| -> String { format!("        (URN::FileName({:?}.to_string()), Hierarchy::File((include_bytes!({:?}), \"{content_type}\".to_string()))),\n", path.file_name().unwrap(), path.to_str().unwrap()) };
            match path.extension() {
                Some(ext) => {
                    match ext.to_str().unwrap() {
                        "json" => view_rs.push_str(push_string("application/json").as_str()),
                        "js" => view_rs.push_str(push_string("application/javascript").as_str()),
                        "css" => view_rs.push_str(push_string("text/css").as_str()),
                        "html" => view_rs.push_str(push_string("text/html").as_str()),
                        "png" => view_rs.push_str(push_string("image/png").as_str()),
                        "jpeg" => view_rs.push_str(push_string("image/jpeg").as_str()),
                        "jpg" => view_rs.push_str(push_string("image/jpeg").as_str()),
                        "svg" => view_rs.push_str(push_string("image/svg+xml").as_str()),
                        "map" => view_rs.push_str(push_string("application/json").as_str()),
                        "csv" => view_rs.push_str(push_string("text/csv").as_str()),
                        "ico" => view_rs.push_str(push_string("image/vnd.microsoft.icon ").as_str()),
                        _ => view_rs.push_str(push_string("text/plain").as_str()),
                    }
                },
                None => view_rs.push_str(push_string("text/plain").as_str()),
            }
        } else {
            view_rs.push_str(format!("        (URN::DirName({:?}.to_string()), Hierarchy::Dir(HashMap::from([\n", path.file_name().unwrap()).as_str());
            view_rs.push_str(visit_dirs(path.as_path()).as_str());
            view_rs.push_str("        ]))),\n");
        }
    }
    view_rs
}