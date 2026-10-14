mod blueprint;
mod cli;
mod file_handler;

fn main() {
    match cli::matchCommand() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {e:?}");
            std::process::exit(1);
        }
    }
}
