Mỗi terminal là công dân hạng nhất của MCP (Model Context Protocol): các tác tử bên ngoài lái terminal qua các công cụ MCP của máy chủ — liệt kê terminal, đọc màn hình, chạy lệnh, chờ hoàn thành — cho phép tác tử lập trình bạn đã dùng vận hành shell tương tác thật.

## Các công cụ MCP terminal của máy chủ

Quanh các phiên terminal, một máy khách MCP có thể:

- `list_terminals` — mọi terminal đang mở; `get_terminal` — tiêu đề, cwd và trạng thái ứng dụng của một terminal;
- `get_screen` — văn bản màn hình nhìn thấy hiện tại; `get_scrollback` — các dòng lịch sử gần đây;
- `send_command` — thực thi một dòng lệnh (Enter được thêm); `send_input` — văn bản thô không Enter (xác nhận `y`, điền biểu mẫu); `send_key` — phím có tên (Enter, Ctrl+C, mũi tên);
- `clear_terminal` — xóa bộ đệm; `resize_terminal` — đổi kích thước PTY; `wait_terminal` — chặn đến khi terminal hết bận.

## Địa chỉ hóa terminal: Terminal ID

Các cuộc gọi công cụ nhắm terminal theo Terminal ID của nó. Chuột phải trong terminal → `MCP` → `Copy Terminal ID`, rồi dán ID cho tác tử của bạn để nó lái đúng terminal đó.

![Chuột phải terminal — menu con MCP](images/terminal-mcp-menu.jpg)

## Đăng ký ứng dụng tác tử một cú nhấp

Menu con `MCP` cũng liệt kê các ứng dụng tác tử được phát hiện trên máy chủ (CLI trình soạn thảo v.v.). Nhấp một ứng dụng sẽ gõ một lệnh `mcp add` sẵn sàng vào terminal — **không có dòng mới cuối**; xem lại và tự nhấn Enter để đăng ký dịch vụ MCP của máy chủ này với tác tử đó. Các mục ẩn khi một ứng dụng đã chạy trong phiên, để văn bản không rơi vào UI của nó.

> Vòng điển hình: sao chép Terminal ID cho tác tử → `send_command` khởi động build → `wait_terminal` đến khi xong → `get_screen` đọc kết quả. Cùng với Tự động xác nhận (xem chương terminal), các tác vụ tương tác dài chạy gần như không cần trông nom.
