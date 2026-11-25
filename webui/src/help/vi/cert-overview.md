Máy chủ bảo vệ mọi điểm cuối TLS mà nó cung cấp — giao diện web, các kết nối của máy khách và các cổng được chuyển tiếp — bằng chứng chỉ do **CA gốc riêng tư** của chính nó cấp. Trang `Chứng chỉ & TLS` trong giao diện web là nơi bạn quản lý CA đó, kiểm soát chứng chỉ có hiệu lực với những tên nào, và trao CA cho các máy khách muốn xác minh máy chủ một cách nghiêm ngặt.

## Cách hoạt động

- Máy chủ tạo một CA gốc riêng tư đúng một lần và giữ nó trong suốt vòng đời của bản cài đặt. Mọi thứ đều do nó ký;
- CA ký một **chứng chỉ lá** cho máy chủ với thời hạn 90 ngày, bao phủ các tên máy chủ và địa chỉ IP mà máy khách thực sự dùng để truy cập;
- Khi tên máy chủ, giao diện mạng hoặc các mục SAN của máy thay đổi, chứng chỉ lá được cấp lại tự động — CA gốc không bao giờ thay đổi, nên những máy khách đã tin tưởng nó vẫn tiếp tục tin tưởng máy chủ;
- Chứng chỉ lá mới được thay vào qua một trình phân giải chứng chỉ dùng chung và được tải nóng: các kết nối hiện có không bao giờ bị ngắt, và các bắt tay mới lập tức dùng chứng chỉ mới nhất. Nếu gia hạn thất bại, chứng chỉ trước đó vẫn tiếp tục hoạt động.

![Trang Chứng chỉ & TLS: phía trên là thẻ CA gốc riêng tư, ở giữa là quản lý SAN, phía dưới là chi tiết chứng chỉ lá của máy chủ](images/certificates-page.jpg)

## Những gì có trên trang

1. `CA gốc riêng tư` — tải xuống hoặc sao chép chứng chỉ công khai của CA để phân phối cho các máy khách (và trình duyệt của chúng). Bản tải xuống không bao giờ chứa khóa riêng;
2. `Tên miền & địa chỉ IP (SAN)` — danh sách các tên mà chứng chỉ lá có hiệu lực. Được mô tả ở trang tiếp theo, Quản lý tên miền & SAN;
3. `Chứng chỉ lá của máy chủ` — chứng chỉ đang hoạt động: trạng thái, chủ thể, dấu vân tay SHA-256, khoảng hiệu lực và các mục SAN.

## Chứng chỉ TLS tùy chỉnh

Nếu một chứng chỉ TLS tùy chỉnh được cấu hình cho máy chủ, nó vẫn giữ quyền: chứng chỉ được quản lý trên trang này vẫn tiếp tục được tạo và cập nhật, nhưng không thay thế nhận diện tùy chỉnh đã cấu hình. Trong khi nhận diện tùy chỉnh đang hoạt động, trang hiển thị một thông báo.

## Tại sao tin tưởng CA gốc

Những máy khách chỉ cần mã hóa hoạt động ngay không cần cấu hình. Những máy khách muốn *xác minh* máy chủ — chế độ nghiêm ngặt được mô tả ở trang cuối, Xác minh nghiêm ngặt trên máy khách — cần chứng chỉ công khai của CA gốc, và nó phải được truyền qua một kênh bạn tin tưởng:

> Chỉ tải CA gốc từ giao diện web cục bộ của máy chủ này hoặc qua một đường hầm đáng tin cậy, và xác minh dấu vân tay SHA-256 (hiển thị trên trang này) trước khi bật xác minh nghiêm ngặt ở bất kỳ đâu.

Cùng một CA đó cũng ký các chứng chỉ mà chuyển tiếp cổng sử dụng (xem trang Chuyển tiếp cổng). Khi một thiết bị đã tin tưởng CA gốc, các URL chuyển tiếp sẽ mở mà không có cảnh báo của trình duyệt *kết nối không phải kết nối riêng tư*.
