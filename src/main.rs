mod write;
mod read;
mod meta;

use read::Read;
use write::Write;

fn main() {
    let mut write = Write::from("Cargo.toml", 10, 10);
    write.go();

    let read = Read::from("Cargo.toml.gif");
    read.go();
}
