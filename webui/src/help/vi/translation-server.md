Dịch thuật là một tác vụ LLM theo tính năng, cấu hình một lần trên máy chủ. Mọi yêu cầu dịch từ bất kỳ client nào — tin nhắn trò chuyện, bản nháp commit, vùng chọn của trình soạn thảo và diff — đều đi qua cùng một bộ cài đặt.

## Bật tính năng

Mở mục `LLM` của giao diện Web và tìm thẻ `Dịch thuật`:

1. Bật `Bật dịch bằng LLM`;
2. Chọn `Nhà cung cấp` — danh sách thả xuống liệt kê các addon LLM đang **bật** trong mục `Tiện ích` (provider tích hợp **OpenAI Compatible LLM** phủ hầu hết tình huống);
3. Điền các trường của provider dưới `Cấu hình nhà cung cấp` (với provider tích hợp: `Base URL`, `API Key`, `Model`, tùy chọn `Extra Body`, giới hạn token, `Timeout`);
4. Chọn `Ngôn ngữ đích` mà mọi bản dịch sẽ được viết ra.

Cấu hình được lưu ngay cả khi tính năng đang tắt, nhưng máy chủ chỉ áp dụng sau khi công tắc được bật. Thiếu nó, yêu cầu từ client thất bại với *"translation addon is not configured or disabled"*.

![Thẻ Dịch thuật trong mục LLM — công tắc bật, chọn provider và ngôn ngữ đích](images/translation-server-card.jpg)

## Ngôn ngữ đích

Khác với các tính năng terminal và Git, dịch không có tùy chọn `Tự động` — nó luôn dịch sang một ngôn ngữ rõ ràng trong danh sách (English, 简体中文, 繁體中文, Deutsch, Español, Français, 日本語, 한국어, Nederlands, Polski, Português, Русский, ไทย, Tiếng Việt, العربية, Bahasa Indonesia, Bahasa Melayu, Українська). Cài đặt áp dụng cho cả máy chủ; việc đổi chỉ ảnh hưởng yêu cầu mới. Ngôn ngữ để trống mặc định về tiếng Anh.

![Danh sách thả xuống ngôn ngữ đích](images/translation-server-language.jpg)

## Tái dùng thông tin xác thực của provider

Các nút sao chép/dán cạnh tiêu đề `Cấu hình nhà cung cấp` nhân bản giá trị trường giữa các thẻ tính năng dùng cùng một addon provider — chọn và điền provider một lần (chẳng hạn cho thông điệp commit Git), nhấn **Sao chép** rồi **Dán** vào thẻ Dịch thuật.

## Chọn mô hình

Bản dịch là các prompt ngắn, nhạy cảm độ trễ — mô hình nhanh thắng mô hình suy luận cỡ lớn. **Qwen3.5 9B** sau một endpoint tương thích OpenAI (Ollama, vLLM, llama.cpp) hoặc **DeepSeek V4 Flash** qua API đều chạy tốt. Máy chủ chờ tối đa 60 giây mỗi yêu cầu; mô hình cục bộ chậm có thể cần `Timeout` cao hơn trong các trường provider.

> Chế độ suy nghĩ cộng thêm vài giây độ trễ mà không nâng chất lượng ở đây — tắt nó trong `Extra Body`, ví dụ `{"enable_thinking": false}` cho Qwen hoặc `{"thinking": {"type": "disabled"}}` cho DeepSeek.
