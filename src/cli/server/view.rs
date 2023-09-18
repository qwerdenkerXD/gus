include!(concat!(env!("OUT_DIR"), "/view.rs"));

pub fn get_view_file(subroutes: &String) -> Option<(ViewFile, ContentHeader)> {
    if subroutes.ends_with("/") {
        return None;
    }
    let segments: &mut Vec<&str> = &mut subroutes.split("/").collect();
    let mut view_files: &ViewFiles = &get_view_files();
    while segments.len() > 0 {
        if segments.len() > 1 {
            let entry = view_files.get(&URN::DirName(segments.remove(0).to_string()))?;
            if let Hierarchy::Dir(dir) = entry {
                view_files = dir;
            } else {
                return None;
            }
        } else {
            let entry = view_files.get(&URN::FileName(segments.remove(0).to_string()))?;
            if let Hierarchy::File((file, content_type)) = entry {
                return Some((file, content_type.clone()))
            } else {
                return None;
            }
        }
    }

    None
}