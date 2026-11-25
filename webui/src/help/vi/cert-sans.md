Chứng chỉ lá chỉ có hiệu lực với các tên được liệt kê trong **Tên thay thế chủ thể (SAN)** của nó. Khi máy khách kết nối, tên máy chủ hoặc IP trong URL phải khớp chính xác với một trong các mục đó — không hỗ trợ ký tự đại diện và không hỗ trợ khớp theo tiền tố. Trang này nói về việc quản lý danh sách đó trên máy chủ; việc kiểm tra ở phía máy khách nằm ở trang cuối, Xác minh nghiêm ngặt trên máy khách.

## Tự động phát hiện

Máy chủ liên tục phát hiện những tên mà nó có thể truy cập — các tên máy chủ hiện tại và địa chỉ của các giao diện mạng đang hoạt động — và liệt kê chúng dưới `Tự động phát hiện`. Các mục này đi theo máy và không thể chỉnh sửa thủ công: thêm một giao diện mạng hoặc đổi tên máy chủ, danh sách sẽ cập nhật theo ở lần gia hạn kế tiếp.

## Tên miền và địa chỉ IP bổ sung

Tự động phát hiện không biết tất cả: tên DDNS, bản ghi DNS split-horizon hay một địa chỉ sau NAT đều phải thêm thủ công. Thêm chúng dưới `Tên miền và địa chỉ IP bổ sung`:

1. Nhập tên máy chủ hoặc địa chỉ IP (ví dụ `server.home.arpa` hoặc `10.0.0.8`) rồi nhấn `Thêm`;
2. Lặp lại cho mỗi tên mà máy khách sẽ dùng. Xóa một mục bằng nút thùng rác;
3. Kiểm tra `Xem trước SAN cuối cùng` — chế độ xem gộp của các mục tự động và thủ công mà chứng chỉ kế tiếp sẽ mang;
4. Nhấn `Lưu & cấp lại`. Khi lưu, hệ thống ký một chứng chỉ lá mới 90 ngày với danh sách SAN mới và tải nóng nó — các kết nối hiện có giữ nguyên, và các bắt tay mới lập tức dùng chứng chỉ mới.

## Cấp lại thủ công và trạng thái

Bảng `Chứng chỉ lá của máy chủ` hiển thị chứng chỉ đang hoạt động: huy hiệu trạng thái, chủ thể, dấu vân tay SHA-256, khoảng hiệu lực và các mục SAN. Huy hiệu trạng thái có ý nghĩa:

- `Khả dụng` — một chứng chỉ lá hợp lệ đã được nạp và đang được dùng;
- `Cập nhật thất bại` — lần cấp lại gần nhất thất bại; chứng chỉ trước đó vẫn tiếp tục hoạt động và lỗi được hiển thị trên trang;
- `Chưa tạo` — chưa có CA hay chứng chỉ lá; dùng `Tạo CA & chứng chỉ` để tạo cả hai.

`Cấp lại chứng chỉ lá` ký ngay một chứng chỉ lá mới mà không chờ kích hoạt tự động — hữu ích ngay sau khi đổi tên máy chủ hoặc giao diện, hoặc để áp dụng các thay đổi SAN được thực hiện ngoài giao diện web.

## Phân phối CA gốc

Những máy khách sẽ dùng xác minh nghiêm ngặt cần chứng chỉ công khai của CA gốc. Lấy nó từ thẻ `CA gốc riêng tư`:

- `Tải xuống CA gốc` lưu tệp PEM;
- `Sao chép CA gốc` đặt PEM vào bộ nhớ tạm, sẵn sàng dán vào trường CA của máy khách.

CA gốc không bao giờ thay đổi qua các lần gia hạn chứng chỉ lá hay chỉnh sửa SAN — phân phối một lần là đủ. Trước khi bật xác minh nghiêm ngặt ở bất kỳ đâu, hãy so sánh dấu vân tay SHA-256 của chứng chỉ hiển thị trên trang này với những gì máy khách nhận được (xem trang cuối, Xác minh nghiêm ngặt trên máy khách).

> Bản tải xuống và bộ nhớ tạm chỉ chứa chứng chỉ công khai của CA — không bao giờ chứa khóa riêng. Chỉ trao nó cho những người bạn muốn có thể xác minh máy chủ của bạn.
