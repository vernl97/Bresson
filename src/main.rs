mod app;
mod cli;
mod engine;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let first = args.first().map(String::as_str).unwrap_or("");
    if matches!(
        first,
        "convert" | "scan-prep" | "synth" | "help" | "--help" | "-h"
    ) {
        if let Err(e) = cli::run(args) {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
        return;
    }
    // Optional folder or file to open.
    let open = args.first().map(std::path::PathBuf::from);
    if let Err(e) = app::run(open) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
