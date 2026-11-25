Bên cạnh màn hình desktop, màn hình từ xa có thể phát thiết bị Android — điện thoại thật qua adb và trình mô phỏng AVD. Máy khách hiển thị chúng trong cửa sổ khung điện thoại với nhập cảm ứng, xoay và điều khiển vòng đời trình mô phỏng từ xa.

## Điều kiện trước

Máy chủ cần tìm bộ công cụ Android:

- **SDK Android** (cung cấp `adb`) — cần cho cả thiết bị và trình mô phỏng;
- **Android Studio** — cần để khởi động trình mô phỏng AVD.

Cả hai mặc định tự dò. Nếu việc dò thất bại hoặc công cụ nằm ngoài vị trí chuẩn, đặt `Android SDK directory` / `Android Studio directory` dưới `Remote Desktop` → `Screen streaming tools`; thay đổi có hiệu lực ngay (áp nóng, không cần khởi động lại máy chủ).

## Nguồn thiết bị

Khi đường dẫn sẵn, máy chủ tự liệt kê các nguồn Android có sẵn:

- **Thiết bị thật đã kết nối** — cắm qua USB với gỡ lỗi USB bật; mọi thứ `adb devices` thấy đều phát được;
- **Trình mô phỏng AVD** — trình mô phỏng đang chạy tự hiện; bạn cũng có thể khởi động một AVD chọn theo yêu cầu từ máy khách (không cần tự tay mở trước) và tắt từ xa.

Mỗi nguồn nhận mục riêng trong thanh SCREENS của máy khách.

## Hướng

Nút xoay của cửa sổ trình mô phỏng đặt `user_rotation` hệ thống qua `adb` — hướng hiển thị của chính thiết bị đổi, và cửa sổ vừa lại theo độ phân giải mới, thích ứng cả dọc lẫn ngang.

## Phía máy khách

Mở tab `Debug` trong bảng trái của máy khách desktop và nhấn nút `Android Simulator` trong thanh SCREENS (chỉ hiện khi máy chủ phơi nguồn Android) để mở cửa sổ trình mô phỏng; bộ chọn thiết bị đổi diện mạo khung điện thoại và `Rotate` lật hướng. Xem Sử dụng máy khách.

> Tiêm cảm ứng trên thiết bị thật dùng kênh nhập của adb (Android 4.4+); một số ROM tùy biến của hãng hạn chế nó — khi đó thiết bị chỉ xem được.
