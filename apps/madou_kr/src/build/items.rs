/// ROM addresses for FFD0 item name patches
const FFD0_WRITER_A: usize = 0x3008A4;   // "a/an" item name (battle drops)
const FFD0_WRITER_B: usize = 0x3008BC;   // "this/these" item name (use/check)
const FFD0_HUNGRY_EL: usize = 0x300896;  // "Hungry Elephant" hardcoded
const EN_PTR_TABLE: u32 = 0x00210000;     // FFF8 redirect pointer table
const ITEM_FFF8_OFF: u8 = 0xD0;          // item ID 0 -> FFF8 entry 0xD0

/// Apply all three FFD0 item name patches to ROM.
pub fn patch_ffd0_items(rom: &mut [u8]) {
    // Patch A: 0x3008A4 - "a/an" writer (24 bytes)
    // Redirects from EN tile table to FFF8 pointer table
    let patch_a: [u8; 24] = [
        0xC0, 0x7C, 0x00, 0x3F,              // AND.W  #$003F, D0
        0x06, 0x40, 0x00, ITEM_FFF8_OFF,      // ADDI.W #$00D0, D0
        0xE5, 0x48,                            // LSL.W  #2, D0
        0x45, 0xF9, 0x00, 0x21, 0x00, 0x00,   // LEA    $00210000, A2
        0x24, 0x72, 0x00, 0x00,               // MOVEA.L (A2,D0.W), A2
        0x60, 0x00, 0x00, 0x1A,               // BRA.W  $3008D4
    ];
    rom[FFD0_WRITER_A..FFD0_WRITER_A + 24].copy_from_slice(&patch_a);

    // Patch B: 0x3008BC - "this/these" writer (24 bytes)
    let patch_b: [u8; 24] = [
        0xC0, 0x7C, 0x00, 0x3F,              // AND.W  #$003F, D0
        0x06, 0x40, 0x00, ITEM_FFF8_OFF,      // ADDI.W #$00D0, D0
        0xE5, 0x48,                            // LSL.W  #2, D0
        0x45, 0xF9, 0x00, 0x21, 0x00, 0x00,   // LEA    $00210000, A2
        0x24, 0x72, 0x00, 0x00,               // MOVEA.L (A2,D0.W), A2
        0x60, 0x00, 0x00, 0x02,               // BRA.W  $3008D4
    ];
    rom[FFD0_WRITER_B..FFD0_WRITER_B + 24].copy_from_slice(&patch_b);

    // Patch C: 0x300896 - "Hungry Elephant" hardcoded (6 bytes)
    // FFF8[0xE4] = item_id 20 + 0xD0 = 0xE4
    // Pointer address: 0x210000 + 0xE4 * 4 = 0x210390
    let hungry_ptr_addr: u32 = EN_PTR_TABLE + (20 + ITEM_FFF8_OFF as u32) * 4;
    let patch_c: [u8; 6] = [
        0x24, 0x79,                            // MOVEA.L (xxx).L, A2
        ((hungry_ptr_addr >> 24) & 0xFF) as u8,
        ((hungry_ptr_addr >> 16) & 0xFF) as u8,
        ((hungry_ptr_addr >> 8) & 0xFF) as u8,
        (hungry_ptr_addr & 0xFF) as u8,
    ];
    rom[FFD0_HUNGRY_EL..FFD0_HUNGRY_EL + 6].copy_from_slice(&patch_c);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_a_location_and_content() {
        let mut rom = vec![0u8; 0x400000];
        patch_ffd0_items(&mut rom);

        // Patch A starts with AND.W #$003F, D0
        assert_eq!(&rom[FFD0_WRITER_A..FFD0_WRITER_A + 4], &[0xC0, 0x7C, 0x00, 0x3F]);
        // ADDI.W #$00D0, D0
        assert_eq!(&rom[FFD0_WRITER_A + 4..FFD0_WRITER_A + 8], &[0x06, 0x40, 0x00, 0xD0]);
        // LEA $00210000, A2
        assert_eq!(&rom[FFD0_WRITER_A + 10..FFD0_WRITER_A + 16],
                   &[0x45, 0xF9, 0x00, 0x21, 0x00, 0x00]);
    }

    #[test]
    fn test_patch_b_bra_offset() {
        let mut rom = vec![0u8; 0x400000];
        patch_ffd0_items(&mut rom);

        // Patch B's BRA offset is 0x0002 (jumps to 0x3008D4)
        assert_eq!(&rom[FFD0_WRITER_B + 20..FFD0_WRITER_B + 24],
                   &[0x60, 0x00, 0x00, 0x02]);
    }

    #[test]
    fn test_patch_c_hungry_elephant() {
        let mut rom = vec![0u8; 0x400000];
        patch_ffd0_items(&mut rom);

        // MOVEA.L opcode
        assert_eq!(&rom[FFD0_HUNGRY_EL..FFD0_HUNGRY_EL + 2], &[0x24, 0x79]);

        // Address should be 0x00210390
        let addr = u32::from_be_bytes([
            rom[FFD0_HUNGRY_EL + 2], rom[FFD0_HUNGRY_EL + 3],
            rom[FFD0_HUNGRY_EL + 4], rom[FFD0_HUNGRY_EL + 5],
        ]);
        assert_eq!(addr, 0x00210390, "Hungry Elephant pointer address");
    }

    #[test]
    fn test_patches_dont_overlap() {
        // Verify the three patches don't overlap in ROM space
        let a_end = FFD0_WRITER_A + 24;
        let b_start = FFD0_WRITER_B;
        let c_start = FFD0_HUNGRY_EL;
        let c_end = FFD0_HUNGRY_EL + 6;

        assert!(a_end <= b_start || FFD0_WRITER_A >= FFD0_WRITER_B + 24,
                "patches A and B overlap");
        assert!(c_end <= FFD0_WRITER_A || c_start >= a_end,
                "patches C and A overlap");
    }
}
