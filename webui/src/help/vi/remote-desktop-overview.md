Màn hình từ xa cho phép bạn theo dõi và điều khiển các màn hình của máy chủ từ máy khách desktop theo thời gian thực: desktop của máy được phát vào một cửa sổ phía bạn, với chuột và bàn phím của bạn được tiêm — như thể bạn đang ngồi trước nó. Ngoài các màn hình desktop, trình mô phỏng Android và iOS có thể xem và điều khiển trong cửa sổ khung điện thoại.

![Máy khách desktop — một cửa sổ màn hình từ xa dùng cùng terminal](images/remote-desktop-overview-hero.webm)

## Nó làm được gì

- **Màn hình desktop** — mỗi màn hình vật lý trên máy chủ phát vào cửa sổ máy khách riêng có thể đổi kích thước;
- **Thiết bị / trình mô phỏng Android** — thiết bị kết nối và trình mô phỏng AVD xuất hiện làm nguồn phát, xem và chạm được trong cửa sổ khung điện thoại, kèm xoay và khởi động/tắt trình mô phỏng từ xa;
- **Trình mô phỏng iOS** (máy chủ macOS) — xem trình mô phỏng iOS Xcode trong cửa sổ khung điện thoại với bộ chọn thiết bị và xoay.

## Cách hoạt động

Máy chủ chụp màn hình và mã hóa nó bằng bộ mã hóa, tốc độ khung, trần độ phân giải và bitrate cấu hình được; máy khách giải mã vào các cửa sổ nổi và gửi sự kiện nhập trở lại qua kết nối máy khách-máy chủ được mã hóa sẵn có — không thêm cổng.

Chuẩn bị theo nền tảng khác nhau: Windows chạy máy chủ màn hình làm dịch vụ hệ thống được và cần trình điều khiển hiển thị ảo trên máy headless; Linux dùng sự đồng ý Portal cộng mở khóa phiên; macOS yêu cầu quyền Ghi màn hình. Xem Thiết lập máy chủ; thiết bị Android được đề cập trong Thiết bị Android, và cửa sổ máy khách trong Sử dụng máy khách.
