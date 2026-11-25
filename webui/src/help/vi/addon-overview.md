Addon mở rộng máy chủ với khả năng mới mà không đụng vào lõi: nhà cung cấp LLM, backend nhận dạng giọng nói, bộ cập nhật DNS động, bề mặt công cụ MCP bổ sung và việc phát hiện ứng dụng agent mã — tất cả đều tồn tại dưới dạng addon. Khu vực `Addons` của Web UI quản lý chúng.

## Loại

Mỗi addon khai báo một loại, và khu vực nhóm các thẻ theo đó:

- **Applications** — dạy trình quan sát terminal nhận diện một CLI agent mã (Claude Code, Codex, Kimi Code, Pi và OpenCode được tích hợp sẵn): phát hiện tiến trình, trạng thái bận/rảnh, móc tự động hóa;
- **LLM Providers** — backend hoàn thành trò chuyện mà các tính năng AI lựa chọn (một nhà cung cấp tương thích OpenAI được tích hợp sẵn);
- **Voice / ASR** — backend nhận dạng giọng nói (addon tích hợp `vibe-voice` trỏ tới máy chủ ASR cục bộ được giám sát);
- **Dynamic DNS** — giữ tên miền trỏ tới địa chỉ hiện tại (Cloudflare được tích hợp sẵn);
- **MCP** — bề mặt công cụ MCP bổ sung phục vụ trên máy chủ MCP (EasyEDA Pro được tích hợp sẵn);
- những loại khác rơi vào nhóm **Other**.

## Addon tích hợp và đã tải lên

- Addon **tích hợp** đi kèm chương trình và luôn sẵn sàng. Manifest và mã nguồn của chúng nguyên vẹn — mã nguồn bị ghi đè có thể khôi phục bất cứ lúc nào;
- Addon **đã tải lên** là của bạn: một zip chứa manifest và mã nguồn JavaScript, cài qua nút `Upload Addon`;
- Huy hiệu `Modified` đánh dấu addon mà bạn đã ghi đè mã nguồn.

Mỗi thẻ hiển thị loại, huy hiệu và công tắc bật; mở ra hiện mô tả, tác giả, trường cấu hình và mã nguồn.

![Khu vực Addons — các thẻ nhóm theo loại với huy hiệu và công tắc bật](images/addon-section.jpg)

## Cấu hình

Manifest của addon khai báo các trường cấu hình, và thẻ mở ra biến chúng thành biểu mẫu. Giá trị lưu theo addon và chỉ có hiệu lực khi addon đang bật; trường kiểu mật khẩu luôn được che. Tắt addon giữ nguyên cấu hình — bật lại thì tiếp tục nơi bạn dừng.
