Dịch thuật biến văn bản bạn gặp trong client thành ngôn ngữ mình chọn: phản hồi trò chuyện AI, thông điệp commit, chú thích mã, hay bất kỳ vùng chọn nào trong trình soạn thảo mã hoặc một diff. Công việc do mô hình dịch được cấu hình trên máy chủ đảm nhận — client chỉ gửi văn bản, hiển thị kết quả và lưu đệm để bạn đổi qua lại giữa bản gốc và bản dịch.

## Dịch thuật có ở đâu

- **Bảng Copilot** — phần đầu mỗi tin nhắn (tin của bạn, phản hồi của mô hình và các khối `Đang suy nghĩ...`) đều có biểu tượng dịch. Một cú nhấp thay nội dung tin nhắn bằng bản dịch; nhấp lần nữa khôi phục bản gốc;
- **Bản nháp thông điệp commit** — ô commit trong bảng Git hiện nút dịch ngay khi bản nháp của bạn dài nhiều dòng. Nút này dịch toàn bộ bản nháp tại chỗ, và thêm một cú nhấp đưa câu chữ gốc của bạn trở lại;
- **Cửa sổ chi tiết commit** — rê chuột lên một commit trong lịch sử Git mở cửa sổ chi tiết; biểu tượng dịch cạnh mã băm đổi thông điệp commit sang ngôn ngữ của bạn;
- **Trình soạn thảo mã và xem diff** — chọn bất cứ thứ gì (vùng chọn khối cũng tính), nhấp chuột phải và chọn `Dịch`. Huy hiệu hình quả cầu nhỏ đánh dấu các dòng đã dịch trong rãnh số dòng, và một thẻ bản dịch mở ra cạnh vùng chọn.

## Cách hoạt động

1. Client gửi văn bản tới máy chủ — kèm tên tệp khi văn bản được chọn trong trình soạn thảo hoặc diff, để mô hình biết tệp dùng cú pháp chú thích nào;
2. LLM dịch của máy chủ dịch sang ngôn ngữ đích được cấu hình trong giao diện Web. Định dạng Markdown (tiêu đề, danh sách, bảng, khối mã) được giữ nguyên; trong vùng chọn từ tệp, các ký hiệu chú thích như `//` hay `#` được nhận diện là trang trí và lược bỏ, còn mã và định danh giữ nguyên vẹn;
3. Client hiển thị kết quả dạng Markdown ngay nơi yêu cầu phát sinh — thay tại chỗ cho tin nhắn trò chuyện, hoặc trong thẻ cuộn được cho vùng chọn của trình soạn thảo và diff.

## Những gì bạn cần

- Một máy chủ bật tính năng dịch — cấu hình một lần được mô tả trong *Thiết lập máy chủ* (mục `LLM` của giao diện Web);
- Một client desktop kết nối tới máy chủ đó. Không cần cấu hình gì phía client: mọi bề mặt đều tự động dùng chung cài đặt máy chủ.

Nếu dịch chưa được cấu hình, client hiển thị lỗi từ máy chủ (ví dụ trong thẻ bản dịch) thay vì kết quả.
