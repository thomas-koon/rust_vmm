use crate::io;

pub const BLOCK_SIZE: usize = 2048; // Size of a sector 
const INITIAL_ENTRY_OFFSET: usize = 32;

pub fn get_boot_catalog_location(data: &[u8]) -> Option<u32> 
{

    // Ensure the file is large enough to contain the volume descriptor set
    if data.len() < BLOCK_SIZE {
        println!("File is too small to contain a volume descriptor set.");
        return None;
    }

    // The volume descriptor set starts after the system area (first 16 sectors)
    let vds_offset = 16 * BLOCK_SIZE;

    let vds = &data[vds_offset..];

    let mut offset = 0;

    while offset + BLOCK_SIZE <= vds.len() 
    {
        let descriptor = &vds[offset..offset + BLOCK_SIZE];
        let descriptor_type = descriptor[0];
        
        if descriptor_type == 0 
        {
            println!("Boot Record Volume Descriptor found at offset {}", offset);
            let identifier = &descriptor[1..6]; 
            if identifier != b"CD001" {
                println!("Invalid Boot Record Volume Descriptor.");
                return None;
            }
            // Extract Boot Catalog location
            let boot_catalog_location = u32::from_le_bytes([
                descriptor[71], descriptor[72], descriptor[73], descriptor[74]
            ]);
            println!("Boot Record Volume Descriptor - El Torito Boot Catalog Location: {}", boot_catalog_location);
            return Some(boot_catalog_location);
        } 
        
        if descriptor_type == 255 
        {
            println!("Volume Descriptor Set Terminator found at offset {}", offset);
            break; // End of descriptors
        }
        
        offset += BLOCK_SIZE;
    }

    None
}



pub fn get_boot_img_start_block_and_sector_count(boot_catalog: &[u8]) -> Option<(u32, u16)>
{

    // Check Validation entry header
    if boot_catalog[0] != 0x01
    {
        println!("Validation entry Header ID is not 0x01.");
        return None;
    }

    // Check Validation entry reserved word
    if boot_catalog[2] != 0x00 && boot_catalog[3] != 0x00
    {
        println!("Validation entry reserved word is not all 0.");
        return None;
    }

    // Initial/Default Entry: The second entry in the Boot Catalog
    let boot_entry = &boot_catalog[INITIAL_ENTRY_OFFSET..INITIAL_ENTRY_OFFSET + 32];

    // Validate Boot Indicator
    if boot_entry[0] != 0x88 
    {
        println!("Boot Indicator is not 0x88 for Bootable.");
        return None;
    }

    // Start address of the virtual disk. Uses relative block addressing (RBA)
    let boot_image_start_block = u32::from_le_bytes([boot_entry[8], boot_entry[9], boot_entry[10], boot_entry[11]]);

    // Sector count: number of sectors stored at Load Segment during initial boot procedure
    let sector_count = u16::from_le_bytes([boot_entry[6], boot_entry[7]]);

    Some((boot_image_start_block, sector_count))

}   

pub fn copy_boot_image(data: &[u8], start_block: u32, sector_count: u16, destination: &mut [u8]) -> io::Result<()> 
{
    let start_offset = (start_block as usize) * BLOCK_SIZE;
    let end_offset = start_offset + (sector_count as usize) * BLOCK_SIZE;

    // Check if the destination buffer is large enough
    if destination.len() < (sector_count as usize) * BLOCK_SIZE 
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Destination buffer is too small"));
    }

    // Check if the data slice has enough data
    if end_offset > data.len() 
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not enough data in source buffer"));
    }

    // Copy the data from the source to the destination buffer
    destination.copy_from_slice(&data[start_offset..end_offset]);

    Ok(())
}