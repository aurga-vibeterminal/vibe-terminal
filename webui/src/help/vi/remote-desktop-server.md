Mọi thứ nằm trong mục `Remote Desktop` của Web UI; trang hiển thị các cài đặt áp dụng cho nền tảng của máy chủ.

## Công tắc chính

`Enable Remote Desktop` — một khi bật, các máy khách từ xa đã kết nối có thể xem desktop của máy này và gửi nhập.

## Cài đặt nền tảng

**Windows**

- `Windows Service` — đăng ký máy chủ màn hình làm dịch vụ hệ thống tự khởi động để màn hình từ xa sống sót qua khởi động lại mà không cần phiên đăng nhập. `Install` / `Uninstall` kích hoạt nhắc UAC trên máy chủ (nút hiển thị tiến độ trong lúc chờ); huy hiệu trạng thái phân biệt đang chạy / đã cài / chưa cài.
- `Máy headless (không màn hình)` — không có màn hình nối (hay thức), Windows không giữ desktop chụp được. Trang liên kết tải trình điều khiển mã nguồn mở Virtual Display Driver (ARM64 / x64 tự chọn); cài nó để luôn có desktop ảo.

![Web UI — mục Remote Desktop trên Windows (thẻ dịch vụ và headless)](images/remote-desktop-server-windows.jpg)

**Linux**

- `Unlock on Connect` — tự mở khóa phiên bị khóa khi một luồng bắt đầu (nơi môi trường hỗ trợ);
- chụp màn hình qua Portal; `Reset Permission` trên thẻ `Screen Recording Permission` xóa sự đồng ý Portal đã lưu để bạn chọn màn hình khác;
- `Auto-start on Login` (trong mục `Server`) — trên máy có trình quản lý hiển thị, máy chủ khởi động cùng phiên đồ họa, nên việc chụp kế thừa toàn bộ môi trường desktop; còn trên máy headless nó khởi động lúc bật máy, trước khi có bất kỳ đăng nhập nào. Để tới desktop khi chưa ai đăng nhập cục bộ, hãy bật thêm đăng nhập tự động trong trình quản lý hiển thị (GDM, SDDM, …); để an toàn, khóa màn hình tự động vẫn có thể giữ bật — phiên và máy chủ vẫn chạy trong lúc khóa, và `Unlock on Connect` (hoặc `loginctl unlock-session`) mở phiên từ xa.
![Web UI — thẻ Screen Recording Permission (Linux)](images/remote-desktop-server-permission.jpg)


**macOS**

- `Screen Recording Permission` — hệ thống yêu cầu phê duyệt tường minh để chụp. Nhấp `Grant Permission` mở hộp thoại hệ thống; `Reset Permission` hồi phục từ trạng thái kẹt.

## Công cụ phát màn hình (hỗ trợ Android)

`Android SDK directory` và `Android Studio directory` định vị adb và trình mô phỏng; để trống cho tự dò. Khi đã đặt, trang hiện đường dẫn được giải và nguồn gốc của nó (auto / env / override); `Reset to Auto-Detect` phục hồi mặc định. Chúng mở khóa gì — xem Thiết bị Android.

## Mã hóa luồng

- `Encoder` — Auto hay một backend cụ thể; máy chủ liệt kê bộ mã dùng được và làm mờ cái không sẵn kèm lý do;
- `Frame rate` — 60 / 30 / 15 fps;
- `Max resolution` — trần phía chụp (4K / 2.5K / 2K);
- `Bitrate` — `Auto` chia theo diện tích với độ phân giải (tốc độ cơ sở tới 1080p, ~4× ở 4K, trần 40 Mbps); giá trị cố định áp ngay cho luồng đang chạy, còn bộ mã / fps / độ phân giải áp cho luồng khởi động sau đó;
- chân thẻ hiển thị bộ mã / fps / độ phân giải / bitrate đang hiệu lực.

![Web UI — cài đặt mã hóa luồng](images/remote-desktop-server-stream.jpg)

Nhấp `Save` để áp thay đổi.
