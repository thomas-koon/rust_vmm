use std::fs::File;
use std::io::{self, Read, Write};
use iso::{BLOCK_SIZE, get_boot_catalog_location, get_boot_img_start_block_and_sector_count, copy_boot_image};

mod v_cpu;
mod iso;

fn main() -> io::Result<()> 
{
    let mut file = File::open("freebsd.iso")?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let boot_catalog_location = match get_boot_catalog_location(&data) 
    {
        Some(location) => location,
        None => 
        {
            println!("Boot catalog location could not be determined.") ;
            return Ok(());
        }
    };

    let boot_catalog_start = boot_catalog_location as usize * BLOCK_SIZE;
    if boot_catalog_start + BLOCK_SIZE > data.len() 
    {
        println!("Boot catalog location is out of bounds.");
        return Ok(()); // or Err(e) if you prefer to handle errors
    }

    let boot_catalog = &data[boot_catalog_start..boot_catalog_start + BLOCK_SIZE];
    let (boot_image_start_block, boot_image_sector_count) = match get_boot_img_start_block_and_sector_count(boot_catalog) 
    {
        Some((start_block, sector_count)) => 
        {
            println!("Boot Image Start Block: {}", start_block);
            println!("Sector Count: {}", sector_count);
            (start_block, sector_count)
        },
        None => 
        {
            println!("Failed to parse El Torito Boot Catalog.");
            (0, 0)
        }
    };

    let mut boot_image = vec![0; (boot_image_sector_count as usize) * BLOCK_SIZE];
    copy_boot_image(&data, boot_image_start_block, boot_image_sector_count, &mut boot_image)?;

    let mut save_file = File::create("bootimg")?;
    save_file.write_all(&boot_image)?;
    println!("Boot image saved to {}", "bootimg");

    Ok(())
}
