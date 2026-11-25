`Host with HTTP server` xuất bản bất kỳ thư mục nào trên máy chủ làm dịch vụ tệp HTTP bằng một cú nhấp: các thiết bị khác trên cùng mạng (hoặc qua relay) — điện thoại, máy tính khác, bộ thử nghiệm — có thể duyệt và tải xuống thư mục đó từ trình duyệt. Lý tưởng để chia sẻ sản phẩm build và gói log, hoặc trao tệp cho thiết bị không cài máy khách.

## Cách dùng

Ba lối vào, đều trên một thư mục: menu chuột phải của danh sách tệp, menu chuột phải của một mục Recent, và menu chuột phải của breadcrumb → `Host with HTTP server`.

Nhấp vào nó:

1. Máy chủ khởi động một dịch vụ HTTP tĩnh gốc tại thư mục đó, với cổng lấy từ dải cổng Static HTTP;
2. **Máy chủ cục bộ**: URL `http://127.0.0.1:port/` được sao chép thẳng vào clipboard, kèm thông điệp trạng thái "HTTP server: … (đã sao chép vào clipboard)";
3. **Máy chủ từ xa** (kết nối qua relay): máy khách tự động yêu cầu ánh xạ reverse proxy và sao chép URL chia sẻ可达 từ bên ngoài — hạn chế LAN không phải rào cản.

Mở URL đó trong trình duyệt của bất kỳ thiết bị nào và thư mục duyệt và tải xuống như một danh sách web thường.

## Quản lý dịch vụ đang chạy

Mỗi dịch vụ HTTP cứ chạy cho tới khi dừng:

- Mở mục `Static HTTP` của Web UI; danh sách `Services` hiển thị mọi dịch vụ đang chạy và gốc tài liệu của nó;
- mỗi mục có nút dừng; dải cổng được cấu hình trên cùng trang.

> Dịch vụ là một danh sách tệp chỉ đọc — không có điểm cuối ghi; nội dung thư mục nhìn thấy trực tiếp khi thay đổi, hợp với log và sản phẩm liên tục sinh ra.
