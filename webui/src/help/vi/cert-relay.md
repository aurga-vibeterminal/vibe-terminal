Theo mặc định, máy chủ chia sẻ mã hóa kết nối thượng nguồn tới relay nhưng không xác minh đầu bên kia là ai — tư thế "tin ngay lần dùng đầu" giống như chấp nhận một chứng chỉ tự ký trong trình duyệt. **TLS nghiêm ngặt cho relay** thay đổi điều đó: máy chủ chia sẻ kiểm tra chứng chỉ của relay dựa trên một CA gốc mà bạn cung cấp và từ chối kết nối trừ khi chuỗi, hiệu lực và tên máy chủ đều đạt. Điều này khiến các cuộc tấn công người đứng giữa nhắm vào liên kết relay trở nên thực tế không thể xảy ra, đổi lấy một bước thủ công: trao cho máy chủ chia sẻ CA gốc của relay. Trang này áp dụng cùng ý tưởng với trang trước, Xác minh nghiêm ngặt trên máy khách, cho liên kết relay thượng nguồn của máy chủ chia sẻ.

## Thiết lập ở đâu

Xác minh nghiêm ngặt được cấu hình trên máy chủ chia sẻ — máy mà `Relay Address` của nó trỏ tới relay. Trong giao diện web của máy đó, mở mục `Server`: bên dưới `Relay Secret Key` là hàng `TLS nghiêm ngặt cho relay` hiển thị trạng thái `On` / `Off` hiện tại. Mở rộng nó để thấy:

1. `CA gốc của relay thượng nguồn` — trường nhiều dòng dành cho chứng chỉ công khai CA gốc của relay;
2. Công tắc `Off` / `On` — chỉ có thể chọn `On` khi PEM đọc được thành chứng chỉ hợp lệ.

![Giao diện web — khối TLS nghiêm ngặt cho relay mở rộng trong mục Server: trường CA gốc của relay thượng nguồn với công tắc Off/On và nút Apply](images/cert-relay-strict-tls.jpg)

Các cài đặt relay không bao giờ được lưu giữa chừng khi chỉnh sửa: xem lại các giá trị rồi nhấn `Apply`. Lưu sẽ kết nối lại relay thượng nguồn với cài đặt mới ngay lập tức.

## Đưa CA gốc đến máy chủ chia sẻ

Hai cách, từ cùng một nguồn (giao diện web của chính relay):

- **Dán chuỗi chia sẻ** — nhấp `QR code` trong giao diện web của máy chủ relay. Chuỗi chia sẻ nhúng CA gốc của relay cùng với địa chỉ và khóa, nên dán nó vào trường `Relay Address` của máy chủ chia sẻ cũng tự động điền trường CA (xem Hướng dẫn thiết lập trong mục Relay). Chế độ nghiêm ngặt vẫn tắt cho đến khi bạn xem lại CA và tự bật nó;
- **Sao chép & dán** — nhấn `Sao chép CA gốc` trên trang `Chứng chỉ & TLS` của relay và dán PEM vào trường `CA gốc của relay thượng nguồn`.

CA gốc giống nhau cho mọi máy chủ chia sẻ và không bao giờ thay đổi qua các lần gia hạn chứng chỉ — cấu hình một lần cho mỗi relay là đủ.

Trước khi tin tưởng CA gốc, hãy xác nhận bạn có bản thật: trang `Chứng chỉ & TLS` của relay hiển thị dấu vân tay SHA-256 của CA gốc. So sánh nó với PEM bạn sắp dán. Nếu khác nhau, đừng bật xác minh nghiêm ngặt — hãy lấy CA qua kênh đáng tin cậy.

## Chế độ nghiêm ngặt kiểm tra những gì

Khi công tắc ở `On`, kết nối `RelayPrimary` thượng nguồn phải trình bày một chứng chỉ mà:

1. Xây dựng chuỗi đến CA gốc đã cấu hình — chứng chỉ do bất kỳ bên nào khác ký, kể cả root hệ thống công cộng, sẽ bị từ chối;
2. Còn trong khoảng hiệu lực — chứng chỉ đã hết hạn hoặc chưa có hiệu lực sẽ bị từ chối;
3. Mang SAN khớp với tên máy chủ của `Relay Address` bạn kết nối tới — chứng chỉ cấp cho các tên khác sẽ bị từ chối.

Một giới hạn cần nhớ: chỉ kết nối thượng nguồn tới relay mới được xác minh. Các kết nối gọi lại của Terminal và yêu cầu giữ nguyên chính sách chứng chỉ hiện có.

Nếu bất kỳ bước kiểm tra nào thất bại, kết nối bị từ chối và được thử lại trong nền với độ trễ tăng dần; máy chủ chia sẻ không thể truy cập qua relay cho đến khi bước kiểm tra đạt hoặc bạn tắt chế độ nghiêm ngặt.

## Các lỗi thường gặp

- `Xác minh TLS nghiêm ngặt cần PEM của CA gốc` — công tắc đang bật nhưng trường CA trống. Dán PEM trước;
- `SAN không khớp` — bạn đang gọi relay bằng tên hoặc IP không có trong chứng chỉ của nó. Thêm tên đó trên trang `Chứng chỉ & TLS` của relay (xem trang Quản lý tên miền & SAN) và cấp lại;
- `Chứng chỉ hết hạn hoặc chưa có hiệu lực` — đồng hồ của relay bị sai, hoặc việc gia hạn chứng chỉ lá thất bại. Kiểm tra trạng thái chứng chỉ trên relay;
- `Không được ký bởi CA đã cấu hình` — CA trong trường thuộc về một relay khác, hoặc kết nối đang bị chặn. Hãy so sánh dấu vân tay trước khi tiếp tục.
