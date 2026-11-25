Nhập bằng giọng nói biến lời thành đầu vào terminal: nói thay vì gõ, kèm trau chuốt LLM tùy chọn cho bản chép trước khi gửi. Nhận dạng chạy như một máy chủ ASR cục bộ với mô hình trên thiết bị — âm thanh và văn bản ở lại máy bạn. Trang này bao phía máy chủ; trải nghiệm desktop nằm ở Sử dụng máy khách.

## Thẻ Local ASR Server

Mở mục `Voice` của Web UI và bật nhận dạng giọng nói. Thẻ `Local ASR Server` quản lý tiến trình vterm-voice được giám sát:

- `Load on Startup` — khởi động máy chủ ASR cùng backend;
- `Bind Address` (`127.0.0.1` chỉ localhost / `0.0.0.0` mọi giao diện) và `Port` (mặc định `8398`) — nơi điểm cuối ASR lắng nghe;
- `Model Directory` + `Local Model` — chọn từ danh mục tích hợp: các dòng Qwen3-ASR, Fun-ASR và Parakeet, mỗi dòng có biến thể full / 8-bit / 4-bit. `Download` lấy mô hình từ Hugging Face hay ModelScope vào thư mục với thanh tiến độ trực tiếp; lệnh `git clone` thủ công được hiện như phương án dự phòng, và mô hình sẵn trên đĩa được đánh dấu khả dụng;
- `Device` — backend suy luận (`Auto`, `CPU`, `CUDA`, `Vulkan`, `Metal`), như máy chủ báo;
- `Access Token` — bí mật chia sẻ của điểm cuối; nút xúc xắc tạo một cái;
- `Save` lưu thẻ (các chỉnh sửa vẫn là bản nháp cục bộ cho tới lúc đó), và `Start Server` / `Stop Server` điều khiển tiến trình. Biểu tượng trạng thái trên tiêu đề thẻ phản ánh khả năng tiếp cận và trạng thái mô hình (đã nạp / đang nạp / lỗi).

![Mục Voice — thẻ Local ASR Server với mô hình, thiết bị và token](images/voice-server-local-asr.jpg)

> **Tranh chấp CUDA**: khi một chương trình suy luận CUDA khác đã chiếm GPU (vd. llama.cpp chạy mô hình cục bộ), để vterm-voice cũng dùng `CUDA` có thể làm cả hai chậm đi thảm hại. Khi đó đặt `Device` thành `CPU` — các mô hình ASR gọn này đủ nhanh trên CPU.

## Nhà cung cấp và thử nghiệm

Menu thả nhà cung cấp liệt kê các addon ASR — addon `vibe-voice` tích hợp trỏ về máy chủ được giám sát này, nên chọn nó là cách bảng dùng mô hình cục bộ của bạn.

Khu `Test` thử mọi thứ không cần máy khách: `Record` (mic → dạng sóng → chép) và `Live` (nhận dạng trực tiếp), mỗi cái kèm thời gian và nút sao chép.

![Khu Test — các tab Record và Live](images/voice-server-test.jpg)
