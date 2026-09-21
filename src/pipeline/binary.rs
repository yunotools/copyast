//! Nhận diện file binary trước khi scanner đọc toàn bộ nội dung.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

// Chỉ cần đọc 8 KiB đầu file để nhận diện
const SAMPLE_SIZE: usize = 8 * 1024;

// Kiểm tra file binary và trả lỗi nếu không đọc được file
pub(crate) fn is_binary_file(path: &Path) -> io::Result<bool> {
    let mut file = File::open(path)?;

    let mut buffer = [0_u8; SAMPLE_SIZE];

    // Trait Read cung cấp method read(): std::io::Read;
    // Thay vì đọc toàn bộ file như: std::fs::read(path)
    // --> chỉ tạo buffer 8 KiB:
    let bytes_read = file.read(&mut buffer)?;

    let sample = &buffer[..bytes_read];

    if sample.is_empty() {
        return Ok(false);
    }

    if has_known_binary_signature(sample) {
        return Ok(true);
    }

    if sample.contains(&0) {
        return Ok(true);
    }

    Ok(has_too_many_control_characters(sample))
}

// Kiểm tra một số signature thường gặp của file binary
// Nhiều định dạng file đặt một chuỗi byte đặc biệt ở đầu file
// Chuỗi này thường được gọi là “magic bytes” hoặc “file signature”
fn has_known_binary_signature(bytes: &[u8]) -> bool {
    const SIGNATURES: &[&[u8]] = &[
        // PNG
        // 89 50 4E 47 0D 0A 1A 0A
        b"\x89PNG\r\n\x1a\n",
        // JPEG
        b"\xff\xd8\xff",
        // GIF
        b"GIF87a",
        b"GIF89a",
        // ZIP, JAR, DOCX, APK…
        b"PK\x03\x04",
        // Gzip
        b"\x1f\x8b",
        // Linux executable
        b"\x7fELF",
        // WebAssembly
        b"\0asm",
        // Windows executable
        b"MZ",
        // PDF
        b"%PDF-",
        // SQLite database
        b"SQLite format 3\0",
    ];

    SIGNATURES
        .iter()
        .any(|signature| bytes.starts_with(signature))
}

// Một file có quá nhiều ký tự điều khiển thường là binary
// Không phải file binary nào cũng có signature mà chúng ta biết
// Vì vậy hàm sử dụng một phép đoán
// Trong bảng ASCII, những byte nhỏ hơn 0x20 chủ yếu là ký tự điều khiển:
// Text bình thường hiếm khi có nhiều ký tự này. Tuy nhiên, ba ký tự điều khiển thường xuất hiện hợp lệ trong file text:
//  - b'\n': xuống dòng.
//  - b'\r': carriage return.
//  - b'\t': tab.
//
fn has_too_many_control_characters(bytes: &[u8]) -> bool {
    let suspicious_byte_count = bytes
        .iter()
        .filter(|&&byte| byte < 0x20 && !matches!(byte, b'\n' | b'\r' | b'\t'))
        .count();

    // Tính % byte điều khiển xuất hiện
    suspicious_byte_count * 100 / bytes.len() > 10
}
