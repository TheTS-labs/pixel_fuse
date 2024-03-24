mod write;
mod read;
mod meta;

use read::Read;
use write::Write;

fn main() {
    let mut write = Write::from("example", 100, 100);
    write.go();

    let read = Read::from("example.gif");
    read.go();
}
