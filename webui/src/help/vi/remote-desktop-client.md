Máy khách desktop mở màn hình từ xa từ thanh SCREENS trên đầu bảng Debug. Mỗi màn hình sống trong cửa sổ nổi riêng dùng được cùng terminal.

## Mở thanh SCREENS

1. Kết nối tới máy chủ đích (trực tiếp hoặc qua relay);
2. nhấp tab `Debug` trong thanh hoạt động trái (biểu tượng công cụ);
3. thanh `SCREENS` hiện trên đầu bảng: một nút hiển thị mỗi màn hình desktop vật lý trên máy chủ, theo sau là các công tắc `iOS Simulator` và `Android Simulator`.

![Bảng Debug — thanh SCREENS với các nút hiển thị và công tắc giả lập](images/remote-desktop-client-screens.jpg)

## Cửa sổ màn hình desktop

Nhấp một nút hiển thị để mở cửa sổ màn hình đó:

- cửa sổ đổi kích thước tự do (tối thiểu 800×600) và hình ảnh xếp lại theo;
- máy chủ đa màn hình nhận một cửa sổ mỗi màn; vài cái có thể mở cùng lúc;
- khi đang kết nối, cửa sổ hiện chỗ trống "đang chuẩn bị desktop"; nếu máy chủ thiếu quyền chụp, cửa sổ hiện trực tiếp hướng dẫn quyền của nền tảng;
- nhấn lại nút thanh hay đóng cửa sổ để ngừng xem (trạng thái nút giữ đồng bộ với cửa sổ).

## Cửa sổ trình mô phỏng iOS

Nhấp `iOS Simulator` để mở cửa sổ khung điện thoại cỡ cố định (264×640 dọc):

- bộ chọn thiết bị phía trên đổi diện mạo điện thoại; cửa sổ vừa theo độ phân giải chọn;
- `Rotate` lật dọc/ngang (cửa sổ 524×380 khi ngang);
- cần máy chủ macOS đang chạy trình mô phỏng iOS Xcode.

## Cửa sổ trình mô phỏng Android

`Android Simulator` hoạt động y hệt: khung điện thoại cố định, bộ chọn thiết bị, nút xoay. Nguồn phát được gồm trình mô phỏng AVD đang chạy và thiết bị thật kết nối; AVD cũng có thể khởi động hay tắt theo yêu cầu từ máy khách (xem Thiết bị Android).

> Các cửa sổ phát máy chủ localhost tự động bị loại khỏi chụp màn hình để tránh gương phản hồi; cửa sổ phát máy từ xa vẫn chụp được để bản ghi bên ngoài còn thấy chúng.
