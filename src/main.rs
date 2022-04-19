use glory_tunnel::Config;

fn main() {
    match Config::from_args() {
        Ok(config) => {
            if let Err(e) = config.run() {
                println!("{}", e)
            }
        }
        Err(e) => println!("{:?}", e),
    }
}
