use glory_tunnel::{Config, Script};

fn main() {
    if let Some(args) = Config::from_args() {
        let script = Script::from(args);

        if let Err(e) = script.run() {
            println!("{}", e)
        }
    } else {
        println!("Invalid Arguments")
    }
}
