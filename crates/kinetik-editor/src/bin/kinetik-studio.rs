//! Kinetik Studio launcher.

fn main() {
    if let Err(error) = kinetik_editor::run_editor_shell() {
        eprintln!("failed to run Kinetik Studio: {error}");
        std::process::exit(1);
    }
}
