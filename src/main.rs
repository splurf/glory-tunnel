use glory_tunnel::Config;

fn main() {
    if let Some(config) = Config::from_args() {
        if let Err(e) = config.run() {
            println!("{}", e)
        }
    } else {
        println!("Invalid Arguments")
    }
}
