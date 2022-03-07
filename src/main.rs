mod vfs;
mod srvfs;

fn main() {
    let nc = nats::connect("127.0.0.1").unwrap();
    srvfs::mount("/tmp/srvfs", nc);
}
