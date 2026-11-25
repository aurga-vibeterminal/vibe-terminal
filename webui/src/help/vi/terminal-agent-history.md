Khi bạn chạy một tác tử mã (Claude Code, Codex, Kimi, Pi, OpenCode…) trong terminal, cuộc hội thoại của nó bị nhốt trong cửa sổ terminal đó — cuộn giữa các đầu ra lệnh, mất đi khi màn hình tiến. `Lịch sử hội thoại tác tử mã` nâng toàn bộ cuộc hội thoại lên một bảng riêng: mỗi lượt trao đổi hiển thị có cấu trúc, các cuộc hội thoại cũ chuyển được, thông điệp dịch được — và bạn có thể gửi lệnh vào terminal ngay từ bảng.

## Mở bảng

Nhấn nút trò chuyện (`AI`) ở bên phải thanh tiêu đề để bật/tắt bảng Copilot. Menu thả xuống phía trên liệt kê **mọi cuộc hội thoại cũ** của tác tử đang chạy trong phiên terminal hoạt động, mới nhất trước; chọn một cái sẽ nạp nó.

![Bảng Copilot — menu thả hội thoại và dòng thông điệp](images/terminal-agent-history.jpg)

## Cách hoạt động

- Máy chủ biết ứng dụng tác tử nào đang chạy trong terminal và phân tích **tệp phiên gốc** của nó (định dạng JSONL/lịch sử của từng tác tử dưới thư mục dự án của nó); thay đổi tệp trực tiếp, giữ cho chế độ xem hội thoại cập nhật;
- Tác tử được hỗ trợ: `claude` / `codex` / `kimi` / `pi` / `opencode`; các ứng dụng khác chưa có bảng hội thoại;
- Thông điệp mang vai trò (bạn / tác tử) và loại (văn bản / suy luận **Thinking**), kèm tên mô hình và thời gian, kết xuất dạng Markdown.

## Tính năng của bảng

- **Chuyển hội thoại**: menu thả `Conversations` phía trên di giữa các phiên cũ, mỗi mục hiện tóm tắt và thời gian;
- **Dịch**: bất kỳ thông điệp tác tử nào cũng có thể dịch theo yêu cầu; bản dịch thay thế văn bản tại chỗ (và có thể bật về lại);
- **Gửi lệnh trực tiếp**: gõ vào hộp dưới cùng (Shift+Enter cho dòng mới) và nhấn Enter — lệnh đi thẳng vào terminal hiện tại, như gõ vào hộp nhập riêng của tác tử mà không chiếm chế độ xem terminal.

## Phối hợp với tính năng khác

- Bảng và terminal là hai góc nhìn của cùng một phiên: câu hỏi của tác tử trong terminal vẫn được trả lời ở đó (hoặc qua Tự động xác nhận); bảng để xem lại lịch sử, không để chặn tương tác;
- Kết hợp với `Terminal MCP` (trang trước), các tác tử bên ngoài cũng có thể đọc màn hình terminal và chạy lệnh — hội thoại hiển thị ở đây cùng hành động MCP tạo thành hồ sơ trọn vẹn công việc của tác tử.
