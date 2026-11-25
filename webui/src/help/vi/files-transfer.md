Truyền tệp giữa máy bạn và máy chủ là kéo & thả, và hướng chính là ý nghĩa: **kéo vào = tải lên máy chủ, kéo ra = tải xuống máy này**. Việc truyền đi trên kết nối máy khách-máy chủ được mã hóa; máy chủ từ xa và chia sẻ ứng xử như cục bộ.

## Tải lên: kéo vào

Kéo tệp (hoặc cả thư mục) từ trình quản lý tệp hệ điều hành lên danh sách tệp và thả:

- ở chế độ cây, **thả lên một hàng thư mục** tải lên thư mục đó; **thả ở gốc** (dòng chèn giữa các hàng) tải lên thư mục gốc hiện tại;
- một hàng truyền hiện dưới danh sách hiển thị `Uploading`, tên tệp và thanh tiến độ;
- hoàn tất hiện `Uploaded n file(s)`; thất bại kèm lý do (vd. từ chối quyền) và danh sách làm mới để lộ tệp mới.

## Tải xuống: kéo ra

Nhấn một tệp trong danh sách, kéo nó ra khỏi cửa sổ máy khách, và thả vào trình quản lý tệp hệ điều hành (hoặc desktop):

- tệp trực tiếp từ máy chủ tới nơi bạn thả;
- các hàng truyền hiển thị tiến độ tương tự; **một hàng tải xuống hoàn tất có thể nhấp** và mở thư mục tải xuống cục bộ;
- chọn nhiều và kéo vài tệp cùng lúc.

![Tiến độ truyền — các hàng tải lên/tải xuống](images/files-transfer.jpg)

> Kéo & thả di chuyển tệp đơn giữa máy này và một máy chủ; để di chuyển dữ liệu giữa hai máy chủ, kết hợp Xuất bản bằng máy chủ HTTP hoặc lệnh terminal (scp/rsync).
