Vibe Terminal là một trung tâm terminal được quản lý từ xa cho phát triển nhúng và máy chủ. Một **máy chủ** chạy trên mỗi máy bạn làm việc — máy phát triển, máy build, bo mạch nhúng — và **các máy khách** kết nối tới nó từ ứng dụng desktop hoặc dòng lệnh. Hướng dẫn này bao trùm cả hai phía: Web UI cấu hình máy chủ, và các máy khách làm việc với nó.

## Cách mọi thứ kết hợp với nhau

- Mọi thứ chạy trên **máy chủ** — các phiên terminal sống ở đó, nên đóng máy khách, mất mạng hay sắp xếp lại cửa sổ không bao giờ làm gián đoạn lệnh đang chạy; kết nối lại và tiếp tục ngay chỗ đã dừng;
- Máy khách quay một kết nối WebSocket được mã hóa duy nhất (`wss://host:port`; `wss://localhost:5153` với máy chủ cục bộ) và xác thực bằng **khóa bí mật** của máy chủ; ghép nối trao khóa qua mã QR hoặc chuỗi chia sẻ được mã hóa (xem Kết nối máy khách);
- **Web UI** là trang cấu hình của chính máy chủ — hồ sơ, addon, nhà cung cấp LLM, relay, chuyển tiếp cổng và hơn thế nữa. Nó không phải terminal: công việc hằng ngày diễn ra trong máy khách desktop hoặc `vtermc`.

## Thành phần

- **Máy chủ** — chạy trên máy đích; chứa các phiên terminal và phục vụ tệp, Git, màn hình từ xa, giọng nói và các tính năng AI cho máy khách đã kết nối;
- **Máy khách desktop** — không gian làm việc native: terminal với chia ô, nhóm tab và cửa sổ tách rời, trình khám phá và trình soạn thảo tệp, công cụ Git, nhập bằng giọng nói, lịch sử hội thoại của tác tử mã, màn hình từ xa và gỡ lỗi từ xa;
- **`vtermc` CLI** — khởi chạy và gắn vào phiên từ bất kỳ terminal thường nào, qua SSH hoặc trên máy headless; chia sẻ danh sách máy chủ với máy khách desktop (xem Máy khách dòng lệnh);
- **Web UI** — cài đặt trên trình duyệt cho chính máy chủ, được tổ chức thành các mục trong thanh bên trái (xem Sử dụng Web UI).

## Biểu tượng khay hệ thống

Trên các máy có phiên desktop, máy chủ hiển thị một biểu tượng khay hệ thống trong lúc chạy:

![Menu biểu tượng khay hệ thống — `Open` đưa máy khách đang chạy lên phía trước hoặc khởi chạy một cái mới, `Settings` mở Web UI, và mục giọng nói khởi động hoặc dừng máy chủ giọng nói](images/start-tray-menu.jpg)

- **Open** — đưa một máy khách desktop đang chạy lên phía trước, hoặc khởi chạy một cái mới khi chưa có cái nào đang chạy (nhấp đúp vào biểu tượng cũng làm tương tự);
- **Settings** — mở Web UI của máy chủ trong trình duyệt (xem Sử dụng Web UI);
- **Start/Stop Vibe Voice** — khởi động hoặc dừng máy chủ giọng nói mà không cần mở Web UI, phản chiếu các nút khởi động/dừng của mục `Giọng nói`; nó xuất hiện khi nhận dạng giọng nói được bật, và nhãn thay đổi theo trạng thái (vd. `Stop Vibe Voice (model on device)` khi mô hình đã nạp);
- **Exit** — tắt máy chủ.

Khi các thư mục đang được lưu trữ qua HTTP, mục `HTTP Servers` cho phép dừng từng chia sẻ chỉ với một cú nhấp (xem Xuất bản bằng máy chủ HTTP).

## Bản đồ tính năng

Phần còn lại của hướng dẫn này phản chiếu các chương trong thanh bên:

- **Thiết bị đầu cuối** — tạo, chia và tổ chức terminal; hồ sơ, WSL và cổng nối tiếp UART; lệnh nhanh và biến môi trường; điều khiển terminal từ tác tử bên ngoài bằng MCP; xem lại hội thoại của tác tử mã; tự động xác nhận lời nhắc tương tác;
- **Tệp** — dự án gần đây, trình khám phá tệp phía máy chủ, tải lên & tải xuống, và xuất bản một thư mục qua HTTP (xem Trình khám phá tệp, Tải lên & tải xuống và Xuất bản bằng máy chủ HTTP);
- **Git** — staging, commit có trợ giúp AI, worktree, rebase tương tác, giải quyết xung đột và push;
- **Màn hình từ xa & trình mô phỏng** — xem và điều khiển màn hình của máy chủ, thiết bị Android và trình mô phỏng, cùng trình mô phỏng iOS;
- **Nhập bằng giọng nói** — nhận dạng giọng nói phía máy chủ với nhập liệu bằng giọng nói push-to-talk và trau chuốt bằng LLM;
- **Relay** — tiếp cận các máy nằm sau NAT hay IP động qua một điểm vào công khai duy nhất;
- **Truy cập từ xa** — miền DDNS và chuyển tiếp cổng TLS cho các dịch vụ chạy trên máy chủ;
- **Tính năng AI** — nhà cung cấp LLM dưới dạng addon, gán theo từng tính năng, cùng tích hợp MCP (xem LLM và MCP).

## Bắt đầu từ đâu

1. Mở mục `Server` của Web UI, đặt `Secret Key` và ghép nối một máy khách — Kết nối máy khách hướng dẫn từng bước thiết lập bằng mã QR và thủ công;
2. Duyệt các mục của Web UI để cấu hình hồ sơ, addon và tính năng — xem Sử dụng Web UI;
3. Mở một terminal trên máy chủ và làm việc theo các chương ở trên khi cần.
