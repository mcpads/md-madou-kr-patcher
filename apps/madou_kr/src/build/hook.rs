/// ROM addresses for the hook system
pub const HOOK_POINT: usize = 0x3004D8;     // JMP to hook (replaces andi.l)
pub const HOOK_ADDR: u32 = 0x0033F000;       // Hook code location
pub const RETURN_ADDR: u32 = 0x00300510;     // Hook returns here
pub const KR_WIDTH_TABLE: u32 = 0x0033F100;  // Korean width table
pub const KR_FONT_BASE: u32 = 0x00340000;   // Korean font data
pub const KR_INDEX_START: u16 = 0x0100;      // Korean tile index start

/// NOP fill range (original code replaced by hook)
pub const NOP_RANGE_START: usize = 0x3004DE;
pub const NOP_RANGE_END: usize = 0x300510;

/// Emit a 16-bit big-endian word to the code buffer.
fn emit_word(code: &mut Vec<u8>, w: u16) {
    code.push((w >> 8) as u8);
    code.push((w & 0xFF) as u8);
}

/// Emit a 32-bit big-endian long to the code buffer.
fn emit_long(code: &mut Vec<u8>, l: u32) {
    code.extend_from_slice(&l.to_be_bytes());
}

/// Assemble the 68K hook code.
///
/// The hook checks if the character tile index (d0) is >= 0x0100.
/// - If < 0x0100: EN path (width from 0x320D46, font from 0x300D46)
/// - If >= 0x0100: KR path (width from KR_WIDTH_TABLE, font from KR_FONT_BASE)
///   Both paths return to RETURN_ADDR (0x300510).
pub fn assemble_hook() -> Vec<u8> {
    let mut code: Vec<u8> = Vec::new();

    // ---- Restore original instruction ----
    // andi.l #$0000FFFF, d0    ; 6 bytes
    emit_word(&mut code, 0x0280);
    emit_long(&mut code, 0x0000FFFF);

    // ---- Korean check ----
    // cmpi.w #$0100, d0        ; 4 bytes
    emit_word(&mut code, 0x0C40);
    emit_word(&mut code, KR_INDEX_START);

    // bcc.s korean_path         ; 2 bytes (branch if d0 >= 0x0100)
    let en_path_size: u8 = 56;
    emit_word(&mut code, 0x6400 | en_path_size as u16);

    // ==== EN path (56 bytes) ====
    let en_start = code.len();

    // asl.l #1, d0              ; char * 2
    emit_word(&mut code, 0xE388);
    // move.l #$320D46, a1       ; EN width table
    emit_word(&mut code, 0x227C);
    emit_long(&mut code, 0x00320D46);
    // adda.l d0, a1
    emit_word(&mut code, 0xD3C0);

    // ---- Width reading code (copy of 3004E8-300502) ----
    emit_word(&mut code, 0x1428); emit_word(&mut code, 0x000C);   // move.b $C(a0), d2
    emit_word(&mut code, 0x1628); emit_word(&mut code, 0x0006);   // move.b $6(a0), d3
    emit_word(&mut code, 0x9403);                                  // sub.b d3, d2
    emit_word(&mut code, 0x1142); emit_word(&mut code, 0x000D);   // move.b d2, $D(a0)
    emit_word(&mut code, 0x1429); emit_word(&mut code, 0x0000);   // move.b 0(a1), d2
    emit_word(&mut code, 0x1142); emit_word(&mut code, 0x0006);   // move.b d2, $6(a0)
    emit_word(&mut code, 0x1229); emit_word(&mut code, 0x0001);   // move.b $1(a1), d1
    emit_word(&mut code, 0x2F01);                                  // move.l d1, -(sp)

    // ---- EN font data ----
    emit_word(&mut code, 0xE188);                       // asl.l #8, d0 -> char * 512
    emit_word(&mut code, 0xE388);                       // asl.l #1, d0 -> char * 1024
    emit_word(&mut code, 0x227C);                       // move.l #$300D46, a1
    emit_long(&mut code, 0x00300D46);
    emit_word(&mut code, 0xD3C0);                       // adda.l d0, a1
    emit_word(&mut code, 0x4EF9);                       // jmp (RETURN_ADDR).l
    emit_long(&mut code, RETURN_ADDR);

    let en_size = code.len() - en_start;
    assert_eq!(en_size, en_path_size as usize, "EN path size mismatch");

    // ==== KR path ====
    // subi.w #$0100, d0         ; kr_index
    emit_word(&mut code, 0x0440);
    emit_word(&mut code, KR_INDEX_START);
    // asl.l #1, d0              ; kr_index * 2
    emit_word(&mut code, 0xE388);
    // move.l #KR_WIDTH_TABLE, a1
    emit_word(&mut code, 0x227C);
    emit_long(&mut code, KR_WIDTH_TABLE);
    // adda.l d0, a1
    emit_word(&mut code, 0xD3C0);

    // ---- Width reading code (same as EN) ----
    emit_word(&mut code, 0x1428); emit_word(&mut code, 0x000C);
    emit_word(&mut code, 0x1628); emit_word(&mut code, 0x0006);
    emit_word(&mut code, 0x9403);
    emit_word(&mut code, 0x1142); emit_word(&mut code, 0x000D);
    emit_word(&mut code, 0x1429); emit_word(&mut code, 0x0000);
    emit_word(&mut code, 0x1142); emit_word(&mut code, 0x0006);
    emit_word(&mut code, 0x1229); emit_word(&mut code, 0x0001);
    emit_word(&mut code, 0x2F01);

    // ---- KR font data ----
    emit_word(&mut code, 0xE188);                       // asl.l #8, d0
    emit_word(&mut code, 0xE388);                       // asl.l #1, d0
    emit_word(&mut code, 0x227C);
    emit_long(&mut code, KR_FONT_BASE);
    emit_word(&mut code, 0xD3C0);
    emit_word(&mut code, 0x4EF9);
    emit_long(&mut code, RETURN_ADDR);

    code
}

/// Generate the JMP instruction bytes for the hook point.
/// Returns 6 bytes: 4EF9 + 32-bit address
pub fn jmp_to_hook() -> [u8; 6] {
    let mut bytes = [0u8; 6];
    bytes[0] = 0x4E;
    bytes[1] = 0xF9;
    bytes[2..6].copy_from_slice(&HOOK_ADDR.to_be_bytes());
    bytes
}

/// Apply the hook to ROM data.
/// 1. Write hook code at HOOK_ADDR
/// 2. Write JMP at HOOK_POINT
/// 3. NOP fill the original code range
pub fn apply_hook(rom: &mut [u8]) {
    let hook_code = assemble_hook();
    let hook_addr = HOOK_ADDR as usize;
    rom[hook_addr..hook_addr + hook_code.len()].copy_from_slice(&hook_code);

    let jmp = jmp_to_hook();
    rom[HOOK_POINT..HOOK_POINT + 6].copy_from_slice(&jmp);

    // NOP fill (0x4E71)
    let mut addr = NOP_RANGE_START;
    while addr < NOP_RANGE_END {
        rom[addr] = 0x4E;
        rom[addr + 1] = 0x71;
        addr += 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_hook_size() {
        let code = assemble_hook();
        // Hook should fit within 128 bytes (0x33F000-0x33F07F)
        assert!(code.len() <= 128, "hook code too large: {} bytes", code.len());
    }

    #[test]
    fn test_assemble_hook_starts_with_andi() {
        let code = assemble_hook();
        // First instruction: andi.l #$0000FFFF, d0 = 0280 0000 FFFF
        assert_eq!(&code[0..2], &[0x02, 0x80]);
        assert_eq!(&code[2..6], &[0x00, 0x00, 0xFF, 0xFF]);
    }

    #[test]
    fn test_jmp_instruction() {
        let jmp = jmp_to_hook();
        assert_eq!(jmp[0], 0x4E); // JMP opcode
        assert_eq!(jmp[1], 0xF9);
        // Address should be HOOK_ADDR (0x0033F000)
        assert_eq!(&jmp[2..6], &[0x00, 0x33, 0xF0, 0x00]);
    }

    #[test]
    fn test_apply_hook_to_rom() {
        let mut rom = vec![0u8; 0x400000];
        apply_hook(&mut rom);

        // Check JMP at hook point
        assert_eq!(&rom[HOOK_POINT..HOOK_POINT + 2], &[0x4E, 0xF9]);

        // Check NOP fill
        for addr in (NOP_RANGE_START..NOP_RANGE_END).step_by(2) {
            assert_eq!(rom[addr], 0x4E, "NOP high byte at 0x{addr:06X}");
            assert_eq!(rom[addr + 1], 0x71, "NOP low byte at 0x{addr:06X}");
        }

        // Check hook code exists
        let hook_addr = HOOK_ADDR as usize;
        assert_ne!(&rom[hook_addr..hook_addr + 6], &[0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_en_kr_paths_both_return() {
        let code = assemble_hook();
        // Both EN and KR paths should end with JMP RETURN_ADDR
        // Check that RETURN_ADDR bytes appear in the code
        let ret_bytes = RETURN_ADDR.to_be_bytes();
        let mut found = 0;
        for window in code.windows(4) {
            if window == ret_bytes {
                found += 1;
            }
        }
        assert_eq!(found, 2, "expected 2 JMP returns (EN + KR paths)");
    }
}
