use std::fs::File;
use std::io::{self, Read};
use iso::{get_boot_catalog_location};

mod iso;

fn main() -> io::Result<()> 
{
    let mut file = File::open("freebsd.iso")?;

    // Read the entire file into a buffer
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    get_boot_catalog_location(&data);

    Ok(())
}