UART nối tiếp biến bảng điều khiển nối tiếp của thiết bị (cổng gỡ lỗi, board nhúng, SBC) thành một tab terminal mà bạn có thể theo dõi và tương tác như mọi terminal khác.

## Cấu hình cổng

Trong mục `UART` của Web UI → `Add UART Port`:

- `Name` — hiển thị làm tiêu đề terminal (vd. "Debug Console");
- `Serial Port` — chọn từ các cổng được phát hiện (số lượng hiển thị ở đầu trang);
- `Baud Rate` / `Data Bits` / `Parity` / `Stop Bits` / `Flow Control` — khớp firmware thiết bị;
- `Terminal Type` và `Encoding` — mã hóa mặc định Auto (UTF-8) và có thể theo firmware;
- `Show Advanced Options` lộ thêm tham số nối tiếp.

Cổng đã lưu xuất hiện trong danh sách `Configured UART Ports` và có thể sửa hay xóa bất cứ lúc nào.

![Trang cài đặt UART — biểu mẫu cổng](images/terminal-uart-config.jpg)

## Sử dụng trong máy khách

Các cổng đã cấu hình mở như tab terminal giống phiên máy chủ thường:

- dữ liệu nối tiếp hiển thị khi tới; nhập bàn phím đi thẳng ra cổng;
- menu chuột phải có `Clear` (xóa màn hình) và menu con `Encoding` (đổi mã hóa trực tiếp; cái hiện tại được đánh dấu);
- phiên UART không có tính năng shell (không có Lệnh nhanh, Apps hay mục menu Envs).

Dụng điển hình: bảng điều khiển router/switch, log board nhúng, giám sát jig nạp firmware.
