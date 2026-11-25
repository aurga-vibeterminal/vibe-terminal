Kết nối một máy khách tới máy chủ này bằng mã QR ghép nối từ Web UI — hoặc bằng cách gõ chi tiết bằng tay. Mọi thứ máy khách cần (địa chỉ, cổng, khóa bí mật) đều hiển thị trong mục Server.

## Chuẩn bị máy chủ

- Mở mục `Server` và đảm bảo `Secret Key` đã được đặt; nhấn nút xúc xắc để tạo một khóa. Tạo lại khóa sẽ vô hiệu mọi máy khách đã ghép nối, chúng phải ghép nối lại.
- Mở rộng khối `QR Code` và chọn `Network Interface` và `IP Version` (IPv4 / IPv6) mà các máy khác nên dùng để truy cập máy chủ này.
- Nếu đã cấu hình miền DDNS, bạn có thể chuyển `Connection Type` sang `DDNS Domain` để mã QR nhúng miền thay vì địa chỉ IP.
- `Pairing Preview` hiển thị URL kết nối chính xác (`wss://host:port`) mà máy khách sẽ quay tới.
- Nhấp vào `QR code` sao chép chuỗi chia sẻ được mã hóa vào clipboard — máy khách dùng nó để tự điền bằng một chạm.

## Máy khách desktop

- Nhấn nút tên máy chủ trên thanh tiêu đề và chọn `Servers` trong menu để mở hộp thoại Server Manager.

![Máy khách desktop — menu máy chủ mở bằng nút tên máy chủ](images/start-connect-server-menu.jpg)

- Nhấn nút `+` bên cạnh danh sách máy chủ để bắt đầu biểu mẫu Add Server.
- Nếu chuỗi chia sẻ nằm trong clipboard, một biểu tượng clipboard hiện ở góc trên phải của biểu mẫu — nhấp vào để giải mã chuỗi và tự điền `Title`, `Address` và `Secret Key`.
- Nếu không, điền các trường bằng tay: tên hiển thị, URL kết nối (`wss://host:port`, địa chỉ IPv6 trong ngoặc vuông) và khóa bí mật.
- Nhấp `Save`, rồi chọn máy chủ mới để mở một phiên.

![Máy khách desktop — Server Manager với biểu mẫu Add Server](images/start-connect-add-server.jpg)
