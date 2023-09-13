mod model;

fn main() {
    if let Err(_) = model::parse_models("./models") {
        println!("Warning: No models defined in ./models");
    }
}