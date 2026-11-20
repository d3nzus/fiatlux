mod blueprint;
mod cli;
mod file_handler;

fn main() {
    match cli::match_command() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}
