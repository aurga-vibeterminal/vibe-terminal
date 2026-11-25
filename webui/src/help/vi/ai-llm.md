Mục `LLM` của Web UI gán một nhà cung cấp LLM cho từng tính năng AI — phân loại terminal, thông điệp commit Git, giải quyết xung đột, trau chuốt văn bản giọng nói và dịch. Chính nhà cung cấp là addon loại LLM được quản lý trong mục `Addons`.

## Nhà cung cấp là addon

Nhà cung cấp LLM sống trong nhóm `LLM` của mục `Addons`:

- Một nhà cung cấp **OpenAI Compatible LLM** được tích hợp; nhà cung cấp tùy chỉnh có thể tải lên dạng zip (nút `Upload`). Manifest addon khai báo các trường cấu hình (khóa API, URL gốc, mô hình, …) mà Web UI biến thành biểu mẫu;
- một nhà cung cấp chỉ xuất hiện trong bộ chọn tính năng khi addon của nó **được bật** — bật trước trong mục Addons;
- nút bút chì trên thẻ addon sửa nguồn của nó; một addon tích hợp bạn đã sửa có thể hoàn về bằng `Reset`.

![Mục Addons — nhà cung cấp LLM là addon có thể bật, sửa hay tải lên](images/ai-llm-addons.jpg)

## Cấu hình theo tính năng

Mỗi thẻ tính năng trong mục `LLM` có công tắc bật riêng và chọn nhà cung cấp riêng, nên các công việc khác nhau dùng mô hình khác nhau:

- **Terminal Classification** — phân loại trạng thái terminal khi các mẫu tích hợp chưa đủ. `Max Consecutive Failures` tắt LLM cho phiên sau bấy nhiêu thất bại liên tiếp (tối thiểu 3);
- **Git Commit Messages** — tạo thông điệp commit từ thay đổi đã stage;
- **Git Conflict Resolve** — giải xung đột rebase/gộp từng hunk một;
- **Voice Text Polish** — trau chuốt bản chép nhập giọng nói trước khi gửi;
- **Translation** — dịch văn bản sang ngôn ngữ đích được chọn.

Mở thẻ để chọn `Provider` của nó và điền các trường nhà cung cấp. Cấu hình được lưu ngay cả khi tính năng tắt và chỉ hiệu lực khi bật. Khi nhiều tính năng dùng cùng một addon nhà cung cấp, các nút sao chép/dán cạnh `Provider Configuration` cho phép nhân bản giá trị trường của nó giữa chúng (xem bên dưới).

Các tính năng Terminal và Git đề nghị ngôn ngữ đầu ra với tùy chọn `Auto` (mô hình quyết định); dịch dùng danh sách đích English/简体中文 tường minh.

![Mục LLM — mỗi tính năng chọn nhà cung cấp và thông tin xác thực riêng](images/ai-llm-features.jpg)

## Sao chép/dán giữa các tính năng

Mỗi tính năng tự lưu một bản sao các trường của nhà cung cấp, nên dùng một nhà cung cấp cho nhiều tính năng thường nghĩa là phải nhập lại cùng một URL gốc, khóa API và mô hình trong từng thẻ. Phần tiêu đề `Provider Configuration` có hai nút nhỏ giúp loại bỏ sự lặp lại:

- **Sao chép** — sao chép các giá trị trường đang hiển thị trong thẻ này, cùng addon nhà cung cấp mà chúng thuộc về;
- **Dán** — chỉ xuất hiện trong một thẻ khi nhà cung cấp được chọn ở đó là cùng addon với cấu hình đã sao chép; nhấn vào để điền các giá trị đã sao chép vào các trường của thẻ.

Để dùng chung một điểm cuối giữa, chẳng hạn, thông điệp commit Git và dịch: chọn nhà cung cấp và điền các trường của nó trong thẻ đầu tiên, nhấn **Sao chép**, mở thẻ còn lại (đã chọn cùng nhà cung cấp) rồi nhấn **Dán**. Các giá trị dán vào được lưu trong cấu hình riêng của tính năng đó như bất kỳ sửa đổi trường nào khác và được lưu cùng với nó. Bảng nhớ (clipboard) chỉ có trong trang — nó bị xóa khi bạn rời mục LLM.

![Nút Sao chép và Dán trong tiêu đề Provider Configuration — nhân bản các trường của một nhà cung cấp giữa các tính năng dùng cùng addon](images/ai-llm-duplicate-config.jpg)

## Nhà cung cấp tương thích OpenAI

Nhà cung cấp tích hợp hoạt động với mọi API trò chuyện tương thích OpenAI. Trường chính:

- `Base URL` / `API Key` / `Model` — điểm cuối, thông tin xác thực, tên mô hình;
- `Extra Body` — một đối tượng JSON gộp vào mỗi thân yêu cầu, cho tham số riêng của mô hình;
- `Max Input Tokens` / `Max Output Tokens` — ngân sách lời nhắc và trần hoàn tất;
- `Timeout` — thời gian chờ mỗi yêu cầu tính bằng giây; tăng cho mô hình cục bộ chậm.

## Mô hình khuyến nghị

Các công việc AI này là lời nhắc nhỏ nhạy độ trễ — một mô hình nhanh "vừa vặn" thắng một mô hình suy luận lớn:

- **Cục bộ: Qwen3.5 9B** — điểm ngọt cho tạo thông điệp commit Git, phân loại terminal, dịch và trau chuốt giọng nói. Phục vụ nó bằng điểm cuối tương thích OpenAI (Ollama, vLLM, llama.cpp) và trỏ `Base URL` tới đó;
- **API: DeepSeek V4 Flash (`deepseek-v4-flash`)** — đủ nhanh cho dùng tương tác và gánh được mọi tính năng trên.

> Chế độ suy nghĩ thêm vài giây độ trễ và hiếm giúp các tác vụ ngắn này — tắt nó trong `Extra Body` cho trải nghiệm tốt hơn, vd. `{"enable_thinking": false}` cho Qwen hoặc `{"thinking": {"type": "disabled"}}` cho DeepSeek (khóa chính xác tùy bộ phục vụ của bạn; JSON được gộp nguyên văn vào thân yêu cầu).
