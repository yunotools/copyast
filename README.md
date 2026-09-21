# Copyast

Copyast quét một hoặc nhiều tệp/thư mục mã nguồn, sau đó gộp các tệp văn bản
thành một tệp duy nhất. Bạn có thể gửi tệp kết quả cho AI để AI đọc và hiểu
ngữ cảnh của một hay nhiều dự án mà không phải tải từng tệp lên riêng lẻ.

Copyast có thể chạy như một chương trình dòng lệnh (CLI) độc lập hoặc như một
tiện ích mở rộng (plugin) của Yuntuns.

## Copyast có thể làm gì?

- Nhận nhiều tệp và thư mục đầu vào trong cùng một lần chạy.
- Quét đệ quy toàn bộ thư mục con của từng đầu vào.
- Gộp nội dung các tệp văn bản UTF-8 vào một tệp ngữ cảnh duy nhất.
- Ghi đường dẫn tệp trước mỗi phần nội dung để AI nhận biết cấu trúc dự án.
- Tự động áp dụng `.gitignore`, `.copyastignore` và nhiều tệp cấu hình bỏ qua phổ biến.
- Bỏ qua tệp nhị phân, tệp không phải UTF-8, tệp không đọc được và tệp quá lớn.
- Phát hiện ngôn ngữ, framework, nội dung trùng lặp và ước lượng số token.
- Chỉ ghi lại kết quả khi dự án thay đổi với chế độ `--incremental`.

Copyast chỉ đọc mã nguồn và không sửa các tệp nguồn. Tệp kết quả sẽ được tạo mới
hoặc ghi đè nếu đã tồn tại.

## Bắt đầu nhanh

Gộp mã nguồn trong thư mục hiện tại vào `copyast-output.txt`:

```bash
copyast -i . -o ./copyast-output.txt
```

Hoặc dùng toàn bộ giá trị mặc định:

```bash
copyast
```

Trong hai lệnh trên:

- `.` là thư mục hiện tại cần quét.
- `./copyast-output.txt` là tệp nhận kết quả.

Sau khi lệnh hoàn tất, hãy gửi `copyast-output.txt` cho AI cùng với câu hỏi của bạn.

Gộp nhiều nguồn vào cùng một tệp:

```bash
copyast --input ./frontend --input ./backend --input ./README.md --output ./project-context.txt
```

Bạn có thể dùng dạng ngắn `-i` và `-o`:

```bash
copyast -i ./frontend -i ./backend -o ./project-context.txt
```

Nếu chưa cài lệnh `copyast`, bạn có thể chạy trực tiếp từ mã nguồn bằng cách thay
`copyast` bằng `cargo run --`:

```bash
cargo run -- -i . -o ./copyast-output.txt
```

## Cài đặt Copyast

### Cài đặt lệnh độc lập

Tại thư mục chứa mã nguồn Copyast, chạy:

```bash
cargo install --path .
```

Kiểm tra cài đặt:

```bash
copyast --version
copyast --help
```

### Cài đặt bằng Yuntuns

Cài đặt từ crates.io:

```bash
yuntuns plugin install copyast
```

Cài đặt từ mã nguồn trên máy để phát triển:

```bash
yuntuns plugin install copyast --path .
```

Yuntuns cũng có thể cài một tệp chương trình đã được biên dịch sẵn:

```bash
cargo build --release
yuntuns plugin install copyast --binary ./target/release/copyast
```

Trên Windows, hãy dùng `copyast.exe` trong lệnh cuối cùng.

Sau khi cài đặt, chạy plugin bằng tên `copyast`:

```bash
yuntuns copyast -i . -o ./copyast-output.txt
```

Các tùy chọn nhiều đầu vào cũng được chuyển tiếp theo cách tương tự:

```bash
yuntuns copyast -i ./frontend -i ./backend -o ./project-context.txt
```

## Cú pháp sử dụng

```text
copyast [-i <PATH> ...] [-o <FILE>] [OPTIONS]
copyast --gen-ignore <TEMPLATE> [-o <FILE>]
copyast --list-templates
```

### Đường dẫn đầu vào và đầu ra

| Tham số | Công dụng | Giá trị mặc định |
| --- | --- | --- |
| `-i <PATH>`, `--input <PATH>` | Thêm một tệp hoặc thư mục đầu vào. Có thể lặp lại tùy chọn này bao nhiêu lần tùy ý. | Thư mục hiện tại (`.`) nếu không dùng `-i` |
| `-o <FILE>`, `--output <FILE>` | Chọn tệp nhận nội dung đã gộp. Nếu đây là một thư mục đã tồn tại, Copyast sẽ tạo `copyast-output.txt` bên trong thư mục đó. | `./copyast-output.txt` |

Copyast tự tạo các thư mục cha của tệp đầu ra nếu cần. Nếu tên đường dẫn có dấu
cách, hãy đặt đường dẫn trong dấu nháy, ví dụ:

```bash
copyast -i "./du an cua toi" -o "./ket qua/ai-context.txt"
```

### Sử dụng nhiều đầu vào

Dùng `-i/--input` nhiều lần để thêm nhiều tệp hoặc thư mục. Tất cả tệp tìm thấy
sẽ được gộp chung vào một tệp kết quả:

```bash
copyast -i ./frontend -i ./backend -i ./shared/config.json -o ./all-context.txt
```

Trong đó:

- Mỗi `-i` thêm một tệp hoặc thư mục cần quét.
- `-o` chọn tệp nhận kết quả đã gộp.
- Có thể kết hợp tệp và thư mục trong cùng một lệnh.
- Nếu các thư mục đầu vào chồng lấp nhau, cùng một tệp vật lý chỉ được gộp một lần.
- Nếu hai tệp khác đường dẫn nhưng có cùng nội dung, chúng vẫn được giữ lại. Dùng
  thêm `--deduplicate` nếu muốn loại các bản sao nội dung này.

### Các tùy chọn quét và tạo kết quả

| Tùy chọn | Công dụng |
| --- | --- |
| `--path-mode <MODE>` | Chọn cách hiển thị đường dẫn trong tệp kết quả. Các giá trị hợp lệ là `auto`, `relative` và `absolute`. |
| `--token-model <MODEL>` | Chọn cách ước lượng token. Các giá trị hợp lệ là `cl100k`, `o200k`, `claude` và `gemini`. Mặc định là `o200k`. |
| `--incremental` | Chỉ ghi lại tệp kết quả khi nội dung nguồn hoặc cấu hình ảnh hưởng đến kết quả đã thay đổi. |
| `--deduplicate` | Với các tệp có nội dung giống hệt nhau, chỉ giữ lại tệp đầu tiên trong kết quả. |
| `--dry-run` | Chỉ quét và in báo cáo; không tạo tệp kết quả và không cập nhật bộ nhớ đệm của `--incremental`. |
| `--no-ignore` | Không áp dụng `.gitignore`, `.copyastignore` và các tệp cấu hình bỏ qua khác. |
| `--hidden` | Bao gồm cả tệp và thư mục ẩn. Mặc định, các mục ẩn sẽ bị bỏ qua. |
| `--ignore-file <NAME>` | Đọc thêm một loại tệp cấu hình bỏ qua. Có thể dùng tùy chọn này nhiều lần. |
| `--max-file-size <BYTES>` | Chỉ đọc các tệp có kích thước không vượt quá số byte đã cho. Mặc định là 10 MiB (`10485760` byte). |
| `-q`, `--quiet` | Ẩn các thông báo thông thường và chỉ hiển thị lỗi. |
| `-h`, `--help` | Hiển thị hướng dẫn nhanh của chương trình. |
| `-V`, `--version` | Hiển thị phiên bản Copyast đang sử dụng. |

#### Cách chọn `--path-mode`

| Giá trị | Đường dẫn được ghi trong kết quả |
| --- | --- |
| `auto` | Dùng đường dẫn tương đối nếu tất cả đầu vào là đường dẫn tương đối. Nếu có ít nhất một đầu vào tuyệt đối, tất cả tiêu đề sẽ dùng đường dẫn tuyệt đối để giữ định dạng nhất quán. Đây là giá trị mặc định. |
| `relative` | Ưu tiên đường dẫn tương đối tính từ thư mục gốc chung của các đầu vào. Kết quả ngắn và vẫn phân biệt được từng thư mục nguồn. |
| `absolute` | Ghi đường dẫn đầy đủ trên máy, ví dụ `C:\projects\app\src\main.rs`. |

Nếu các đầu vào nằm trên những ổ đĩa khác nhau và không có thư mục gốc chung,
Copyast sẽ dùng đường dẫn tuyệt đối để không làm mất thông tin vị trí tệp.

#### Cách chọn `--token-model`

Tùy chọn này chỉ thay đổi số token được **ước lượng trong báo cáo**. Nó không thay
đổi nội dung, không rút gọn và không giới hạn kích thước tệp kết quả.

| Giá trị | Khi nào nên dùng |
| --- | --- |
| `o200k` | Giá trị mặc định; dùng để ước lượng theo họ tokenizer o200k. |
| `cl100k` | Dùng khi cần ước lượng theo họ tokenizer cl100k. |
| `claude` | Dùng khi chuẩn bị ngữ cảnh cho Claude. |
| `gemini` | Dùng khi chuẩn bị ngữ cảnh cho Gemini. |

Số token chỉ là giá trị gần đúng, không phải kết quả đếm chính xác từ dịch vụ AI.

#### Chế độ cập nhật tăng dần

Khi dùng `--incremental`, Copyast tạo một tệp bộ nhớ đệm bên cạnh tệp kết quả:

```text
copyast-output.txt.copyast-cache
```

Ở những lần chạy sau, nếu nguồn và cấu hình tạo kết quả không thay đổi, Copyast
không ghi đè tệp kết quả. Cách này hữu ích khi bạn chạy Copyast thường xuyên trong
script hoặc công cụ tự động hóa.

## Chọn những tệp cần bỏ qua

Theo mặc định, Copyast áp dụng quy tắc từ `.gitignore`, `.ignore`,
`.copyastignore`, `.dockerignore`, `.npmignore` và nhiều tệp cấu hình bỏ qua phổ
biến khác trong cây thư mục.

Bạn có thể tự tạo `.copyastignore` với cú pháp tương thích với `.gitignore`:

```gitignore
target/
node_modules/
dist/
*.log
.env
```

Thêm một tên tệp cấu hình bỏ qua riêng của dự án:

```bash
copyast -i . -o ./copyast-output.txt --ignore-file .mycompanyignore
```

Có thể dùng `--ignore-file` nhiều lần:

```bash
copyast -i . -o ./copyast-output.txt --ignore-file .mycompanyignore --ignore-file .localignore
```

`--no-ignore` tắt việc đọc các tệp cấu hình bỏ qua, nhưng tệp và thư mục ẩn vẫn
bị bỏ qua. Hãy dùng đồng thời `--no-ignore --hidden` nếu bạn thực sự muốn quét cả
những mục ẩn:

```bash
copyast -i . -o ./copyast-output.txt --no-ignore --hidden
```

### Tạo `.copyastignore` từ mẫu có sẵn

Xem danh sách mẫu ngôn ngữ và framework:

```bash
copyast --list-templates
```

Tạo `.copyastignore` cho dự án Rust:

```bash
copyast --gen-ignore rust
```

Kết hợp nhiều mẫu bằng dấu phẩy hoặc dấu cộng:

```bash
copyast --gen-ignore rust,tauri -o ./.copyastignore
copyast --gen-ignore python+django -o ./.copyastignore
```

Tạo tệp chứa tất cả mẫu:

```bash
copyast --gen-ignore all
```

| Tùy chọn | Công dụng |
| --- | --- |
| `--list-templates` | In danh sách tên mẫu hiện có. |
| `--gen-ignore <TEMPLATE>` | Tạo quy tắc bỏ qua từ một hoặc nhiều mẫu. Nếu không chỉ định `-o`, kết quả được ghi vào `./.copyastignore`. |
| `-o <FILE>`, `--output <FILE>` | Khi dùng với `--gen-ignore`, chọn tệp nhận các quy tắc được tạo. |

Lưu ý: `--gen-ignore` sẽ ghi đè tệp đích nếu tệp đó đã tồn tại. Hai lệnh
`--gen-ignore` và `--list-templates` là thao tác riêng, không dùng chung với
`-i/--input` hoặc các tùy chọn quét.

## Ví dụ thường dùng

### Gộp nhiều dự án hoặc thư mục

```bash
copyast -i ../frontend-app -i ../backend-api -i ../shared-library -o ./full-project-context.txt --deduplicate
```

### Gộp toàn bộ dự án hiện tại

```bash
copyast -i . -o ./copyast-output.txt
```

### Gộp một tệp duy nhất

```bash
copyast -i ./src/main.rs -o ./main-context.txt
```

### Xem trước kết quả quét mà không tạo tệp

```bash
copyast -i . -o ./copyast-output.txt --dry-run
```

### Loại nội dung trùng lặp và chỉ cập nhật khi có thay đổi

```bash
copyast -i . -o ./ai-context.txt --deduplicate --incremental
```

### Giới hạn mỗi tệp nguồn ở mức 1 MiB

```bash
copyast -i . -o ./copyast-output.txt --max-file-size 1048576
```

### Dùng đường dẫn tương đối và ước lượng token cho Claude

```bash
copyast -i . -o ./copyast-output.txt --path-mode relative --token-model claude
```

## Nội dung tệp kết quả

Mỗi tệp nguồn được đặt sau một phần tiêu đề chứa đường dẫn của tệp đó:

```text
=== Yunotools-Copyast ===
=== FILE: src/main.rs ===
<nội dung của src/main.rs>
```

Copyast sắp xếp tệp theo đường dẫn để kết quả ổn định giữa các lần chạy. Tệp đầu
ra đang được tạo, tệp bộ nhớ đệm và các tệp tạm của Copyast cũng tự động bị loại
khỏi quá trình quét.

Khi có nhiều đầu vào và dùng `--path-mode relative`, Copyast tính thư mục gốc
chung để giữ đủ thông tin phân biệt từng nguồn. Ví dụ, hai nguồn `frontend` và
`backend` sẽ tạo các tiêu đề như `frontend/src/app.ts` và
`backend/src/main.rs`.

Sau khi chạy, báo cáo trên màn hình cho biết:

- Số tệp đã sao chép và số tệp bị bỏ qua.
- Lý do bỏ qua: bị chặn bởi quy tắc, là tệp nhị phân/không phải UTF-8, không đọc
  được hoặc vượt quá giới hạn kích thước.
- Tổng dung lượng ngữ cảnh và số token ước lượng.
- Ngôn ngữ và framework được phát hiện.
- Các nhóm tệp có nội dung trùng hoàn toàn.
- Số tệp thay đổi, không thay đổi hoặc đã bị xóa khi dùng `--incremental`.
