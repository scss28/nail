pub fn src_position(src: impl AsRef<[u8]>, index: usize) -> (usize, usize) {
    let mut char = 1;
    let mut line = 1;
    for byte in &src.as_ref()[..index] {
        char += 1;
        if *byte == b'\n' {
            line += 1;
            char = 0;
        }
    }

    (line, char)
}
