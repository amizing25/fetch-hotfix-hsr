/// Converts a buffer of bytes to a string, replacing invalid UTF-8 sequences with replacement characters.
fn buffer_to_string(buffer: &[u8]) -> String {
    String::from_utf8_lossy(buffer).to_string()
}

/// Reads a string from a buffer starting at the given offset. The first byte at the offset represents the length of the string.
pub fn read_string(buffer: &[u8], offset: usize) -> String {
    let length_byte = buffer[offset];
    let end_pos = offset + length_byte as usize + 1;

    buffer_to_string(&buffer[offset + 1..end_pos])
}

/// Strips trailing zero bytes from the buffer.
pub fn strip_empty_bytes(buffer: &[u8]) -> &[u8] {
    buffer
        .iter()
        .rev()
        .position(|&b| b != 0x00)
        .map_or(buffer, |pos| &buffer[..buffer.len() - pos])
}

/// Finds the last occurrence of a delimiter in the buffer and returns the part after it.
pub fn last_index_of(buffer: &[u8], delimiter: u8) -> &[u8] {
    buffer.rsplitn(2, |&b| b == delimiter).next().unwrap_or(&[])
}

/// Reads a 3-byte unsigned integer from the buffer in big-endian order.
pub fn read_uint24_be(buffer: &[u8], offset: usize) -> u32 {
    (buffer[offset] as u32) << 16 | (buffer[offset + 1] as u32) << 8 | buffer[offset + 2] as u32
}

/// Validates if the dispatch seed contains only hexadecimal digits.
fn seed_sanity_check(dispatch_seed: &str) -> bool {
    dispatch_seed.chars().all(|c| c.is_digit(16))
}

/// Splits a buffer by a delimiter and returns a vector of the resulting slices.
pub fn split_buffer(buffer: &[u8], delimiter: u8) -> Vec<&[u8]> {
    buffer
        .split(|&b| b == delimiter)
        .filter(|part| !part.is_empty())
        .collect()
}

/// Searches for a dispatch seed in the buffer slices that starts with the given `constructed_string`.
/// Returns an `Option` containing a tuple of (constructed_string, dispatch_seed) if found.
pub fn get_dispatch_seed(
    buffersplits: &[&[u8]],
    constructed_string: &str,
) -> Option<(String, String)> {
    for i in 1..buffersplits.len() {
        if buffersplits[i].len() < 2 {
            continue;
        }

        let current_string = read_string(buffersplits[i], 0);
        if current_string.starts_with(constructed_string) {
            let seed = read_string(buffersplits[i - 1], 0);
            if seed_sanity_check(&seed) {
                return Some((current_string, seed));
            }
        }
    }
    None
}

/// Opens a file dialog to allow the user to select a folder.
/// Returns the selected folder's path, or `None` if the selection is canceled.
pub fn select_folder() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_directory(".")
        .set_title("Select HSR Folder")
        .pick_folder()
}

/// Returns the path to the "BinaryVersion.bytes" file located under "StarRail_Data/StreamingAssets" from the given base path.
pub fn get_binary_version_path(base: &std::path::PathBuf) -> std::path::PathBuf {
    base.join("StarRail_Data/StreamingAssets/BinaryVersion.bytes")
}

/// Returns the path to the "ClientConfig.bytes" file located under "StarRail_Data/StreamingAssets" from the given base path.
pub fn get_client_config_path(base: &std::path::PathBuf) -> std::path::PathBuf {
    base.join("StarRail_Data/StreamingAssets/ClientConfig.bytes")
}

/// Finds the starting position of the last occurrence of a 9-byte zero-pattern in the buffer.
/// Returns the index after the pattern, or `0` if not found.
pub fn get_last_buffer_start(buffer: &[u8]) -> usize {
    let zero_pattern = vec![0u8; 9];

    buffer
        .windows(zero_pattern.len())
        .rposition(|window| window == zero_pattern)
        .map(|pos| pos + zero_pattern.len())
        .unwrap_or(0)
}
