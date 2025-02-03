use duct::cmd;

fn main() {
    let frontend_dir = "./frontend";

    cmd!("npm", "install").dir(frontend_dir).run().unwrap();
    cmd!("npm", "run", "build").dir(frontend_dir).run().unwrap();
}
