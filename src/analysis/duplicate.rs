//! Tìm và loại các file có nội dung trùng hoàn toàn.

use crate::domain::{DuplicateGroup, TextFile};
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct DuplicateDetector;

impl DuplicateDetector {
    // Tìm những nhóm file có nội dung giống hệt nhau
    pub fn find_duplicates(files: &[TextFile]) -> Vec<DuplicateGroup> {
        find_exact_groups(files)
            .into_iter()
            .map(|file_indices| build_duplicate_group(files, &file_indices))
            .collect()
    }

    // Xóa các file trùng khỏi Vec
    // File đầu tiên trong mỗi nhóm được giữ lại.
    // Function trả về số file đã xóa
    pub fn remove_duplicates(files: &mut Vec<TextFile>) -> usize {
        // tương đương
        // let groups = find_exact_groups(files);
        //
        // let mut duplicate_indices = HashSet::new();
        //
        // for file_indices in duplicate_groups {
        //     for file_index in file_indices.into_iter().skip(1) {
        //         duplicate_indices.insert(file_index);
        //     }
        // }
        let duplicate_indices = find_exact_groups(files)
            .into_iter()
            // Giả sử find_exact_groups() trả về:
            // vec![
            //     vec![0, 2, 5],
            //     vec![1, 4],
            // ]
            // Điều này có nghĩa:
            // File 0, 2, 5 giống nhau
            // File 1, 4 giống nhau
            // Trong mỗi nhóm, ta giữ phần tử đầu tiên:
            // [0, 2, 5] → giữ 0, xóa 2 và 5
            // [1, 4]    → giữ 1, xóa 4
            .flat_map(|file_indices| {
                // Bỏ index đầu tiên vì file đầu tiên
                // trong nhóm sẽ được giữ lại
                file_indices.into_iter().skip(1)
            })
            .collect::<HashSet<_>>();

        let removed_files = duplicate_indices.len();

        let mut file_index = 0_usize;

        // retain() duyệt từng phần tử trong vector:
        // - Closure trả true → giữ phần tử.
        // - Closure trả false → xóa phần tử.
        files.retain(|_| {
            let should_keep = !duplicate_indices.contains(&file_index);

            file_index += 1;

            should_keep
        });

        removed_files
    }
}

// Tìm index của các file trùng nhau
// Ví dụ kết quả:
// [
//     [0, 3, 5],
//     [2, 7],
// ]
//
// Nghĩa là:
// - file 0, 3 và 5 giống nhau;
// - file 2 và 7 giống nhau

// Các file
//    ↓
// Nhóm theo (kích thước, hash)
//    ↓
// So sánh nội dung thật
//    ↓
// Các nhóm duplicate chính xác
fn find_exact_groups(files: &[TextFile]) -> Vec<Vec<usize>> {
    let mut candidate_groups = HashMap::<(u64, u64), Vec<usize>>::new();

    // 1: gom những file có cùng kích thước và hash vào 1 nhóm ứng viên
    for (file_index, file) in files.iter().enumerate() {
        let fingerprint = (file.count_bytes(), hash_content(&file.content));

        candidate_groups
            .entry(fingerprint)
            .or_default()
            .push(file_index);
    }

    let mut duplicate_groups = Vec::new();

    // 2: so sánh nội dung thật để loại trừ trường hợp 2 file vô tình có cùng hash
    for candidate_indices in candidate_groups.into_values() {
        if candidate_indices.len() < 2 {
            continue;
        }

        let content_groups = split_hash_collisions(files, candidate_indices);

        for file_indices in content_groups {
            if file_indices.len() > 1 {
                duplicate_groups.push(file_indices);
            }
        }
    }

    // HashMap không bảo đảm thứ tự.
    // Sort giúp kết quả ổn định giữa các lần chạy
    duplicate_groups.sort_by(|left, right| files[left[0]].path.cmp(&files[right[0]].path));

    duplicate_groups
}

// Tạo fingerprint nhanh cho nội dung file
// Hash này chỉ dùng để tìm ứng viên duplicate,
// không dùng cho mật khẩu hoặc bảo mật.
fn hash_content(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

// Tách những file có cùng hash nhưng nội dung khác nhau
// Hash collision hiếm nhưng vẫn có thể xảy ra
fn split_hash_collisions(files: &[TextFile], candidate_indices: Vec<usize>) -> Vec<Vec<usize>> {
    let mut content_groups = Vec::<Vec<usize>>::new();

    // 'candidate là tên của vòng lặp
    // Đây không phải lifetime dù cú pháp khá giống lifetime
    'candidate: for file_index in candidate_indices {
        for matching_indices in &mut content_groups {
            // Lấy file đầu tiên làm đại diện cho cả group
            let representative_index = matching_indices[0];

            if files[representative_index].content == files[file_index].content {
                matching_indices.push(file_index);
                continue 'candidate;
            }
        }

        // Chưa có nhóm nào có cùng nội dung,
        // tạo một nhóm mới
        content_groups.push(vec![file_index]);
    }

    content_groups
}

// Chuyển nhóm index thành DuplicateGroup
fn build_duplicate_group(files: &[TextFile], file_indices: &[usize]) -> DuplicateGroup {
    let file_paths = file_indices
        .iter()
        .map(|&file_index| files[file_index].path.clone())
        .collect::<Vec<_>>();

    let redundant_copies = u64::try_from(file_indices.len().saturating_sub(1)).unwrap_or(u64::MAX);

    let redundant_bytes = files[file_indices[0]]
        .count_bytes()
        .saturating_mul(redundant_copies);

    DuplicateGroup {
        file_paths,
        redundant_bytes,
    }
}
