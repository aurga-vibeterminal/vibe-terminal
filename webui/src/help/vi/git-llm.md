Trí tuệ của bảng Git do LLM dẫn động: thông điệp commit một cú nhấp, đọc diff có AI trợ giúp, và giải quyết xung đột bởi tác tử từng hunk một. Tất cả dựa trên một nhà cung cấp LLM được cấu hình trong Web UI.

## Cấu hình nhà cung cấp

Mở mục `LLM` của Web UI:

- Chọn một `Provider` và điền thông tin xác thực của nó (các trường do manifest của nhà cung cấp định nghĩa); cấu hình là toàn cục — phân loại terminal, thông điệp commit Git, trợ lý trò chuyện và các tính năng AI khác dùng chung nhà cung cấp;
- Nhà cung cấp là addon loại LLM: các cái phổ biến được tích hợp, và nhà cung cấp tùy chỉnh có thể tải lên dạng zip trong mục `Addons` (manifest khai báo các trường cấu hình mà Web UI biến thành biểu mẫu).

![Mục LLM của Web UI — chọn nhà cung cấp và điền thông tin xác thực](images/git-llm-provider-config.jpg)

## Tính năng LLM trong Git

- **Tạo thông điệp commit** — nhấn nút AI cạnh hộp commit; LLM đọc diff đã stage và viết một thông điệp, được gõ vào hộp với hoạt ảnh. Thông điệp vẫn sửa được — chỉ commit khi bạn ưng ý;
- **Phân tích diff** — nút lấp lánh trên đầu khung diff phân tích diff hiện tại;
- **Tác tử AI xung đột** — chế độ AI của trình soạn thảo gộp rebase giải từng hunk một kèm giải thích và độ tin cậy (xem Giải quyết xung đột).

![Thông điệp commit do AI tạo được gõ vào hộp commit](images/git-llm-commit-message.jpg)

> LLM chỉ đọc nội dung kho để tạo văn bản; commit và gộp luôn xảy ra khi bạn xác nhận chúng.
