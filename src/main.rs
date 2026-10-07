mod file_handler;
mod cli;
mod blueprint;

fn main() {
    match cli::matchCommand() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}
