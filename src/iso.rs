use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

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



fn parse_el_torito_boot_catalog(boot_catalog: &[u8])
{
    // Initial/Default Entry: The second entry in the Boot Catalog
    let boot_entry = &boot_catalog[INITIAL_ENTRY_OFFSET..INITIAL_ENTRY_OFFSET + 32];

    // Start address of the virtual disk. Uses relative block addressing (RBA)
    let boot_image_start_block = u32::from_le_bytes([boot_entry[8], boot_entry[9], boot_entry[10], boot_entry[11]]);

    // Sector count: number of sectors stored at Load Segment during initial boot procedure
}