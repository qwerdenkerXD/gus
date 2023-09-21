// built via build_view.rs build-script
include!(concat!(env!("OUT_DIR"), "/view.rs"));

/*
    get_view_file: 
        Returns the content of the file declared by the uri.

    returns:
        The content of the declared file
        or an Error if there isn't such file
*/
pub fn get_view_file(uri: &String) -> Option<(ViewFile, ContentHeader)> {
    if uri.ends_with("/") {
        return None;
    }
    let segments: &mut Vec<&str> = &mut uri.split("/").collect();
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