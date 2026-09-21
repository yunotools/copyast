//! Ước lượng kích thước context theo từng họ tokenizer AI.

use crate::config::{CopyastConfig, TokenModel};
use crate::domain::TextFile;
use crate::pipeline::writer::render_header;

// Kết quả ước lượng token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenEstimate {
    // Model được sử dụng để ước lượng.
    pub token_model: TokenModel,

    // Tổng token ước lượng.
    pub estimated_tokens: u64,

    // Tổng số ký tự, bao gồm header.
    pub character_count: u64,

    // Tổng số byte UTF-8, bao gồm header.
    pub byte_count: u64,
}

pub struct TokenEstimator;

// characters khác bytes thế nào?
// Với ASCII:
//      let text = "abc";
// Ta có:
//      characters = 3
//      bytes      = 3
// Nhưng với tiếng Việt:
//      let text = "ế";
// Ta thường có:
//      characters = 1
//      bytes      = 3
// Bởi vì Rust lưu String dưới dạng UTF-8. Một ký tự Unicode có thể cần nhiều byte.
impl TokenEstimator {
    // Ước lượng tổng token của tất cả file,
    // bao gồm cả header do Copyast tạo ra.
    pub fn estimate(files: &[TextFile], config: &CopyastConfig) -> TokenEstimate {
        let token_model = config.token_model;
        let mut total_tokens = 0_u64;
        let mut total_characters = 0_u64;
        let mut total_bytes = 0_u64;

        for (file_index, file) in files.iter().enumerate() {
            if file_index > 0 {
                accumulate_text(
                    "\n",
                    token_model,
                    &mut total_tokens,
                    &mut total_characters,
                    &mut total_bytes,
                );
            }

            let header = render_header(file, config);

            accumulate_text(
                &header,
                token_model,
                &mut total_tokens,
                &mut total_characters,
                &mut total_bytes,
            );

            accumulate_text(
                &file.content,
                token_model,
                &mut total_tokens,
                &mut total_characters,
                &mut total_bytes,
            );

            if !file.content.ends_with('\n') {
                accumulate_text(
                    "\n",
                    token_model,
                    &mut total_tokens,
                    &mut total_characters,
                    &mut total_bytes,
                );
            }
        }

        TokenEstimate {
            token_model,
            estimated_tokens: total_tokens,
            character_count: total_characters,
            byte_count: total_bytes,
        }
    }

    fn estimate_text(text: &str, token_model: TokenModel) -> u64 {
        let mut ascii_chars = 0_u64;
        let mut non_ascii_chars = 0_u64;

        for ch in text.chars() {
            if ch.is_ascii() {
                ascii_chars += 1;
            } else {
                non_ascii_chars += 1;
            }
        }

        let ascii_tokens = match token_model {
            TokenModel::Claude => {
                // Khoảng 3.8 ký tự ASCII cho một token
                divide_ceil(ascii_chars.saturating_mul(10), 38)
            }

            _ => {
                // Khoảng 4 ký tự ASCII cho một token.
                divide_ceil(ascii_chars, 4)
            }
        };

        let non_ascii_tokens = match token_model {
            TokenModel::Cl100k => {
                // Tokenizer cũ thường xử lý Unicode
                // kém hiệu quả hơn
                non_ascii_chars
            }

            TokenModel::O200k => {
                // Ước lượng khoảng 2 token
                // cho mỗi 3 ký tự Unicode
                divide_ceil(non_ascii_chars.saturating_mul(2), 3)
            }

            TokenModel::Claude => non_ascii_chars,

            TokenModel::Gemini => {
                // Ước lượng khoảng 3 token
                // cho mỗi 4 ký tự Unicode
                divide_ceil(non_ascii_chars.saturating_mul(3), 4)
            }
        };

        ascii_tokens.saturating_add(non_ascii_tokens)
    }
}

fn accumulate_text(
    text: &str,
    token_model: TokenModel,
    total_tokens: &mut u64,
    total_characters: &mut u64,
    total_bytes: &mut u64,
) {
    *total_tokens = total_tokens.saturating_add(TokenEstimator::estimate_text(text, token_model));
    *total_characters = total_characters.saturating_add(count_chars(text));
    *total_bytes = total_bytes.saturating_add(count_bytes(text));
}

// Chia số nguyên nhưng làm tròn lên.
// Ví dụ:
// 5 / 4 bình thường bằng 1.
// Hàm này trả về 2.
fn divide_ceil(value: u64, divisor: u64) -> u64 {
    if value == 0 {
        return 0;
    }

    value.saturating_add(divisor - 1) / divisor
}

fn count_chars(text: &str) -> u64 {
    let count = text.chars().count();
    u64::try_from(count).unwrap_or(u64::MAX)
}

fn count_bytes(text: &str) -> u64 {
    let count = text.len();
    u64::try_from(count).unwrap_or(u64::MAX)
}
