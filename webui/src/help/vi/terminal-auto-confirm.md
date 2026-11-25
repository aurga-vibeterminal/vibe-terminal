Bộ phân tích terminal phía máy chủ theo dõi đầu ra terminal: nó phát hiện dấu nhắc lệnh, nhận biết hoàn thành tác vụ, và với LLM phân loại những gì terminal đang hiển thị. `Auto Confirm` là tính năng được dùng nhiều nhất — khi terminal dừng ở dấu nhắc xác nhận thông thường (như `Proceed? [Y/n]`), Enter được gửi thay bạn, nên các tác vụ dài không còn kẹt ở câu hỏi.

## Cấu hình

Mục `Terminal Analysis` của Web UI:

- `Terminal observer` — công tắc chính của phân tích phía máy chủ;
- `Auto confirm` — chỉ gửi Enter cho dấu nhắc được phân loại là **xác nhận thông thường**; các câu hỏi khác không bao giờ được trả lời mù quáng;
- `Analysis delay (seconds)` — chờ bao lâu sau khi tác tử nhàn rỗi trước khi phân tích, tránh các cuộc gọi LLM dư thừa giữa các tác vụ;
- `Confirmation grace (seconds)` — khoảng thời gian bạn có thể tiếp quản sau khi phát hiện xác nhận;
- `Notify on completion` — phát một sự kiện khi đầu ra được phân loại là tác vụ hoàn tất (nạp vào thông báo).

> Ghi chú trên trang: khi chế độ tự động của chính tác tử đang bật, nên tắt `Auto Confirm` ở máy khách để hai tự động hóa không trả lời đè lên nhau.

## Phụ thuộc LLM

Việc phân loại do một LLM thực hiện: chọn `Provider` và điền thông tin xác thực của nó trong mục `LLM` (dùng chung bởi phân loại terminal, thông điệp commit Git, trợ lý trò chuyện và các tính năng AI khác). Không có LLM, bộ phân tích vẫn hoạt động với độ chính xác giảm — trong trường hợp đó giữ tự động xác nhận tắt.

![Trang cài đặt Terminal Analysis](images/terminal-auto-confirm.jpg)

![Trang cài đặt LLM — chọn nhà cung cấp và điền thông tin xác thực](images/terminal-auto-confirm-llm.jpg)
