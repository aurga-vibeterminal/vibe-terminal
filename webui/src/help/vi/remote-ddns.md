Internet gia đình và văn phòng thường đi kèm IP công khai động đổi mà không báo trước. DDNS giữ một miền cố định trỏ tới IP hiện tại của máy — máy khách và các máy chủ khác chỉ cần nhớ tên, không bao giờ địa chỉ.

## Cách hoạt động

![Kiến trúc DDNS — trình quản lý định kỳ dò IP cục bộ, addon cập nhật bản ghi DNS qua API nhà cung cấp, máy khách kết nối bằng miền](diagram/ddns)

Cả đường ống chạy tự động trong nền máy chủ:

1. **Trình quản lý DDNS** làm mới theo chu kỳ cố định (mặc định mỗi 5 phút, chỉnh qua biến môi trường `TERMINAL_SERVER_DDNS_POLL_INTERVAL_SECS`); lưu cấu hình kích hoạt làm mới tức thì, và không gì chạy khi dịch vụ bị tắt;
2. **Dò IP cục bộ** đọc giao diện mạng được chọn trước, dự trữ về dò theo tuyến, và lọc kết quả theo cài đặt `IP Family` (IPv4 / IPv6 / kép);
3. **Addon DDNS** — một script JS cách ly — nhận cấu hình đã giải mã và IP hiện tại rồi gọi API nhà cung cấp DNS qua HTTP. Addon Cloudflare tích hợp chẳng hạn tìm vùng theo miền và ghi lại bản ghi A / AAAA thành IP hiện tại, xóa bản ghi cũ khi một họ địa chỉ biến mất;
4. **Máy khách** phân giải miền ra IP mới nhất và kết nối bằng tên.

## Cấu hình

Mở mục `DDNS` của Web UI:

1. Bật `Enable`. Bạn cũng có thể để dịch vụ tắt trong lúc điền mọi thứ — dịch vụ tắt vẫn lưu cấu hình nhưng không bao giờ chạy;
2. chọn nhà cung cấp dưới `Provider`; `Cloudflare DDNS` được tích hợp;
3. điền các trường nhà cung cấp (ví dụ Cloudflare):
   - `Email` — email tài khoản Cloudflare;
   - `API Token` — token có quyền Zone → DNS → Edit;
   - `Domain` — miền gốc của vùng, vd. `example.com`;
   - `Subdomain` — tên bản ghi trong vùng, vd. `sub` cho `sub.example.com`; để trống cho gốc vùng;
4. trong `Network Interface & IP`, chọn `Interface` và `IP Family`. `Dual Stack` yêu cầu giao diện có cả địa chỉ IPv4 lẫn IPv6;
5. nhấp `Save`, rồi `Sync Now` để kiểm chứng. `Sync Status` hiện IP cuối được đẩy và thời gian đồng bộ, kèm thông báo lỗi của addon khi thất bại.

## Bảo mật

Thông tin xác thực nhà cung cấp (token API) được mã hóa trước khi lưu (`enc-v2:`) và chỉ giải mã trong bộ nhớ khi addon chạy; addon chạy trong runtime JS cách ly với HTTP đi ra làm năng lực duy nhất.

## Mở rộng nhà cung cấp

Nhà cung cấp là addon: một manifest JSON (khai báo các trường cấu hình mà Web UI biến thành biểu mẫu) cộng một script JS lộ `run()`. `cloudflare-ddns` tích hợp là hiện thực tham chiếu — nhà cung cấp tùy chỉnh có thể tải lên dạng zip trong mục `Addons`.

## Cùng các tính năng khác

- Khi miền phân giải được, mã QR của mục `Server` có thể chuyển `Connection Type` sang `DDNS Domain`, để máy khách quét kết nối bằng miền;
- các máy chủ khác có thể đặt `wss://your-domain:port` vào `Relay Address` của chúng để chạy chuỗi relay qua miền (xem chương Relay).
