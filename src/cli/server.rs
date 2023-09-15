pub mod model;

use std::path::Path;

pub fn start(model_path: &Path) {
    if let Err(_) = model::parse_models(model_path) {
        println!("Warning: No models defined in {}", model_path.display());
    }
    todo!();
}