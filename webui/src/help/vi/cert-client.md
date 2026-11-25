Theo mặc định, máy khách mã hóa mọi kết nối nhưng không xác minh đầu bên kia là ai — tư thế "tin ngay lần dùng đầu" giống như chấp nhận một chứng chỉ tự ký trong trình duyệt. **Xác minh nghiêm ngặt** thay đổi điều đó: máy khách kiểm tra chứng chỉ của máy server dựa trên một CA gốc mà bạn cung cấp và từ chối kết nối trừ khi chuỗi, hiệu lực và SAN đều đạt. Điều này khiến các cuộc tấn công người đứng giữa nhắm vào kết nối trở nên thực tế không thể xảy ra, đổi lấy một bước thủ công: trao cho máy khách CA gốc của máy chủ.

## Thiết lập ở đâu

Xác minh nghiêm ngặt được cấu hình cho từng máy server trong bảng chỉnh sửa máy server của máy khách, bên cạnh địa chỉ và khóa bí mật:

1. `Chứng chỉ CA gốc (PEM)` — một trường nhiều dòng dành cho chứng chỉ công khai CA gốc của máy chủ;
2. `Xác minh CA nghiêm ngặt (chuỗi, hiệu lực và SAN)` — chỉ có thể chọn sau khi đã nhập CA.

![Bảng chỉnh sửa máy server trong máy khách desktop: trường nhiều dòng Chứng chỉ CA gốc (PEM) với ô kiểm xác minh nghiêm ngặt bên dưới](images/client-ca-fields.jpg)

## Đưa CA gốc đến máy khách

Hai cách, từ cùng một nguồn (trang `Chứng chỉ & TLS` của máy chủ — xem trang Quản lý tên miền & SAN):

- **Sao chép & dán** — nhấn `Sao chép CA gốc` trong giao diện web và dán PEM vào trường `Chứng chỉ CA gốc (PEM)` của máy khách;
- **Nhập chuỗi chia sẻ** — các chuỗi chia sẻ do máy khách tạo ra nhúng CA gốc cùng với địa chỉ và khóa. Nhập chuỗi chia sẻ đó sẽ tự động điền trước trường CA; người nhận chỉ cần tích ô xác minh nghiêm ngặt.

CA gốc giống nhau cho mọi máy khách và không bao giờ thay đổi qua các lần gia hạn chứng chỉ — cấu hình một lần cho mỗi máy chủ là đủ.

## Chế độ nghiêm ngặt kiểm tra những gì

Khi ô kiểm được bật, mọi kết nối tới máy chủ đó phải trình bày một chứng chỉ mà:

1. Xây dựng chuỗi đến CA gốc đã cấu hình — chứng chỉ do bất kỳ CA nào khác ký sẽ bị từ chối;
2. Còn trong khoảng hiệu lực — chứng chỉ đã hết hạn hoặc chưa có hiệu lực sẽ bị từ chối;
3. Mang SAN khớp với địa chỉ bạn đang kết nối tới — chứng chỉ cấp cho các tên khác sẽ bị từ chối.

Nếu bất kỳ bước kiểm tra nào thất bại, kết nối bị từ chối kèm lỗi cho biết bước nào đã thất bại. Không có ô kiểm (hoặc không có CA), kết nối vẫn được mã hóa nhưng bỏ qua các bước kiểm tra này.

## Xác minh dấu vân tay

Trước khi tin tưởng CA gốc, hãy xác nhận bạn có bản thật: trang `Chứng chỉ & TLS` hiển thị dấu vân tay SHA-256 của CA gốc. So sánh nó với PEM bạn sắp dán (hoặc PEM nhúng trong chuỗi chia sẻ bạn nhận được). Nếu khác nhau, đừng bật xác minh nghiêm ngặt — hãy lấy CA qua kênh đáng tin cậy.

## Các lỗi thường gặp

- `SAN không khớp` — bạn đang kết nối bằng một tên hoặc IP không có trong chứng chỉ. Thêm nó trên máy chủ (xem trang Quản lý tên miền & SAN) và cấp lại;
- `Chứng chỉ hết hạn hoặc chưa có hiệu lực` — đồng hồ máy server bị sai, hoặc việc gia hạn chứng chỉ lá thất bại. Kiểm tra trạng thái chứng chỉ trong giao diện web;
- `Không được ký bởi CA đã cấu hình` — CA trong trường thuộc về một máy chủ khác, hoặc kết nối đang bị chặn. Hãy so sánh dấu vân tay trước khi tiếp tục.
