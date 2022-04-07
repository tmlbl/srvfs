use structopt::StructOpt;

mod vfs;
mod srvfs;

#[derive(StructOpt, Debug)]
#[structopt(name = "srvfs")]
struct Options {
    #[structopt(long = "nats", default_value = "127.0.0.1")]
    nats_addr: String,
}

fn main() {
    let opts = Options::from_args();

    println!("Connecting to NATS at {}...", opts.nats_addr);
    let nc = nats::connect(&opts.nats_addr).unwrap();
    srvfs::mount("/tmp/srvfs", nc);
}
