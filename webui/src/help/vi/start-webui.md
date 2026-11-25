Web UI là trang cấu hình của chính máy chủ: mọi tính năng của máy chủ được sắp xếp thành từng mục, liệt kê trong thanh bên trái. Đây không phải là terminal — công việc hằng ngày diễn ra trên máy khách desktop hoặc `vtermc` (xem Tổng quan). Trang này giải thích bố cục và các hành vi chung; các chương về tính năng đề cập chi tiết cài đặt của từng mục.

## Bố cục

- **Thanh bên** — danh sách các mục (bên dưới), biểu tượng trợ giúp cạnh tên máy chủ, và phiên bản ứng dụng ở cuối; trên màn hình hẹp nó trượt vào từ bên trái và nút hamburger trên tiêu đề dùng để bật tắt nó;
- **Tiêu đề** — tên của mục đang chọn cùng bộ chọn ngôn ngữ và nút chuyển giao diện sáng/tối ở góc trên phải (trên màn hình hẹp biểu tượng trợ giúp cũng nằm ở đó);
- **Nội dung** — biểu mẫu của mục đã chọn, được căn giữa và cuộn được.

## Các mục

Theo thứ tự trên thanh bên:

- `Máy chủ` — cài đặt cổng, chứng chỉ TLS và xác thực: `Secret Key`, `QR Code` ghép nối, địa chỉ relay và công tắc `Auto-start on Login` đều nằm ở đây (xem Kết nối máy khách và Hướng dẫn thiết lập). Trên Linux, gợi ý tự khởi động cũng giải thích đăng nhập tự động của trình quản lý hiển thị để tới desktop trước khi có đăng nhập cục bộ (xem Thiết lập máy chủ trong Màn hình từ xa & trình mô phỏng);
- `Terminal` — phân tích terminal: dò tìm lời nhắc, tự động xác nhận và phân loại LLM (xem Tự động xác nhận);
- `UART` — hồ sơ terminal nối tiếp dựa trên các cổng UART cục bộ (xem Cổng nối tiếp UART);
- `Tiện ích` — các tiện ích đã cài: bật, sửa hoặc tải lên chúng (nhà cung cấp LLM là tiện ích — xem LLM);
- `LLM` — gán một nhà cung cấp LLM cho từng tính năng AI (xem LLM);
- `Giọng nói` — nhà cung cấp nhận dạng giọng nói, mô hình và cài đặt phiên âm (xem Thiết lập máy chủ trong Nhập bằng giọng nói);
- `Môi trường` — các bộ sưu tập biến môi trường xếp thành cây nhóm (xem Biến môi trường);
- `MCP` — máy chủ MCP của terminal mở cho các tác tử mã (xem Terminal MCP);
- `DDNS` — nhà cung cấp DDNS và cài đặt đồng bộ (xem DDNS);
- `Chuyển tiếp cổng` — dải cổng chuyển tiếp và các chuyển tiếp TLS đang hoạt động (xem Chuyển tiếp cổng);
- `Static HTTP` — các thư mục được lưu trữ qua HTTP và các chia sẻ tệp đang hoạt động (xem Xuất bản bằng máy chủ HTTP);
- `Màn hình từ xa` — cài đặt chụp, nhập và phát trực tuyến cho màn hình từ xa (xem Thiết lập máy chủ trong Màn hình từ xa & trình mô phỏng);
- `Thống kê LLM` — mức sử dụng token, số lượng yêu cầu và chỉ số hiệu năng trên mọi tính năng LLM.

## Lưu & mẹo

- Hầu hết các mục tự động lưu ngay sau một lần sửa — chỉ báo đang lưu và mọi lỗi hiện ở cuối mục;
- `MCP`, `DDNS` và `Màn hình từ xa` giữ các thay đổi cho tới khi nhấn `Lưu thay đổi`: khi còn thay đổi chờ lưu, gợi ý `Thay đổi chưa lưu` hiện cạnh các nút, và `Đặt lại` hủy chúng;
- Khi được mở từ máy khách di động, Web UI theo ngôn ngữ và giao diện của ứng dụng và ẩn các điều khiển ngôn ngữ, giao diện của chính nó.
