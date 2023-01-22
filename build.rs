#[path = "src/config_file.rs"]
mod config_file;

fn main() {
    config_file::create_or_get_default();
}