Máy chủ có sẵn một host Model Context Protocol tích hợp, và khu vực `MCP` của Web UI dùng để cấu hình nó: các Code Agent (Claude Code, Codex, Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness, …) có thể kết nối ngay từ chat của mình và làm việc trực tiếp trên máy của bạn — đọc và điều khiển terminal, thậm chí vận hành EasyEDA Pro.

## Cấu hình máy chủ

Bật máy chủ MCP và thiết lập:

- `Port` (mặc định `6180`) và `Bind Address` — `127.0.0.1` giữ endpoint chỉ trên localhost; `0.0.0.0` mở ra mạng;
- `Access Token` — bắt buộc với yêu cầu không từ localhost: bên gọi từ xa phải trình nó như Bearer token, còn yêu cầu từ localhost luôn được chấp nhận. Nó không bao giờ rỗng — bản cài mới kèm một token ngẫu nhiên; nút xúc xắc tạo lại, nút sao chép đưa ra;
- `Stream Throttle` — khoảng cách tối thiểu giữa các lần đẩy luồng màn hình tới các agent đang kết nối.

Thay đổi có hiệu lực khi `Save`; máy chủ tải lại không cần khởi động lại.

## Endpoint

Mỗi bộ năng lực được phục vụ tại URL riêng `http://localhost:{port}/{id}/mcp`:

- **Terminal MCP** (`/terminal/mcp`) — bề mặt terminal tích hợp sẵn. Agent có thể liệt kê terminal, đọc màn hình đang thấy và lịch sử cuộn, gõ đầu vào thô và phím có tên, đổi kích thước và chờ prompt. Lệnh tool nhắm tới một terminal theo Terminal ID của nó — chuột phải vào terminal → `MCP` → `Copy Terminal ID` rồi dán ID vào ngữ cảnh của agent (xem Terminal MCP trong khu vực Terminals);
- **EasyEDA Pro MCP** (`/easyeda/mcp`) — một addon MCP tích hợp sẵn bắc cầu EasyEDA Pro tới agent: một bộ công cụ cố định nhỏ cùng danh mục các lệnh gọi `eda.*` có thể tìm kiếm, chuyển tiếp tới tiện ích mở rộng kết nối EasyEDA qua WebSocket. Hãy bật addon trong khu vực `Addons` trước — endpoint của nó xuất hiện trong danh sách sau khi cài, và huy hiệu cho biết tiến trình addon hiện đã kết nối hay chưa. Trường `Timeout` của addon giới hạn mỗi yêu cầu cầu nối (tăng lên cho các thao tác EDA chậm);
- **Các addon MCP khác** — mọi addon loại MCP đều thêm endpoint của mình theo cách tương tự, kèm huy hiệu đã kết nối/mất kết nối theo thời gian thực.

Chọn CLI agent trong danh sách thả xuống phía trên; nút sao chép cạnh một endpoint sẽ đưa ra lệnh thiết lập chạy được ngay cho ứng dụng đó.

## Kết nối agent

Token không bao giờ phải dán thủ công. Máy chủ duy trì một hồ sơ môi trường `Vibe Terminal MCP Bearer Token` được tạo tự động (xem khu vực Environment Variables) xuất `VTERM_MCP_TOKEN`; áp dụng nó vào terminal trước khi kết nối agent rồi dùng lệnh đã sao chép:

- **Ứng dụng lệnh shell** (Claude Code, Codex) — chạy lệnh `mcp add` đã sao chép trong một terminal thường. Codex đọc token từ `VTERM_MCP_TOKEN` qua `--bearer-token-env-var`, nên bí mật không rơi vào dòng lệnh;
- **Ứng dụng hội thoại** (Kimi Code, Pi, OpenCode, Qwen Code, DeepSeek Harness) — khởi động ứng dụng với hồ sơ đã áp dụng, rồi gửi prompt đã sao chép trong cuộc trò chuyện. Agent sẽ trực tiếp cập nhật cấu hình MCP. Kimi dùng `bearerTokenEnvVar`; Pi, OpenCode và Qwen Code dùng token Bearer từ `VTERM_MCP_TOKEN`, nên token không xuất hiện trong chat.

Trong Pi và Qwen Code, `/mcp` kiểm tra kết nối.

Thẻ `Command Templates` dưới phần cấu hình lưu các lệnh thiết lập theo ứng dụng: sửa mục, ghi đè mục tích hợp hay thêm mẫu cho CLI agent mới — phần xem trước cho thấy đúng những gì nút sao chép sẽ tạo ra.
