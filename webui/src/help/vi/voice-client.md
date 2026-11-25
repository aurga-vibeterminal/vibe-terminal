Nhấn `Ctrl` hai lần liên tiếp nhanh (trong ~300 ms) bất kỳ đâu trong máy khách desktop để mở hộp thoại nhập bằng giọng nói.

## Ghi âm

Hộp thoại bắt đầu ghi ngay: dạng sóng lỏng theo giọng bạn, bộ chọn trên cùng đổi micro (lựa chọn được nhớ), và bộ đếm thời gian hiển thị thời gian đã trôi. `Stop` kết thúc đoạn và gửi đi chép (`Reset` khởi động lại).

![Giai đoạn ghi — dạng sóng, bộ chọn micro và bộ đếm](images/voice-client-record.jpg)

## Xử lý

Trong lúc máy chủ chép âm thanh, một hoạt hình dòng hạt chạy.

## Chỉnh sửa

Bản chép đáp xuống một ô văn bản chỉnh sửa được:

- viên chế độ chuyển giữa `original` và `polished`; `Polish` chạy LLM một lần, và `Auto polish` làm ngay sau mỗi lần chép;
- `Re-record` vứt văn bản và bắt đầu lại; `Insert` ghi lại và nối bản chép mới vào văn bản sẵn có tại con nháy (thay bất kỳ vùng chọn nào), để bạn mở rộng hay sửa những gì đã đọc — con nháy đáp ngay sau đoạn được chèn;
- rê lên ô văn bản để lộ nút sao chép ở góc trên phải của nó: một cú nhấp sao chép văn bản và nó lật thành dấu kiểm; sửa văn bản hay đổi chế độ nạp lại nó cho lần sao chép khác;
- `Submit` (hoặc `Ctrl+Enter`) gõ văn bản vào terminal đang hoạt động; `Esc` đóng hộp thoại.

![Giai đoạn chỉnh — viên chế độ, tự động trau chuốt, hàng hành động với Insert, và nút sao chép khi rê](images/voice-client-editing.jpg)

## Thiết lập trau chuốt

`Polish` cần một LLM trau chuốt giọng nói: trong mục `LLM` của Web UI, bật `Enable voice text polish` trong danh sách theo tính năng (một nhà cung cấp thường cộng công tắc này là đủ). Không có nó, nút `Polish` giữ xám và tùy chọn tự động trau chuốt bị ẩn.
