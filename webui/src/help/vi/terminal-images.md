Ảnh chụp màn hình và terminal gặp nhau trong một dòng chảy: một phím tắt toàn cục duy nhất chụp bất kỳ vùng nào của màn hình, bạn chú thích rồi sao chép hoặc lưu — sau đó dán ảnh từ clipboard thẳng vào terminal, nơi nó đáp xuống thành đường dẫn tệp mà chương trình đang chạy (kể cả tác tử mã) có thể mở và dùng. Cả hai nửa đều sống trong máy khách desktop.

## Chụp màn hình

- Nhấn `Shift`+`Alt`+`A` ở bất kỳ đâu: phím tắt là toàn cục, nó nổ ra bất kể ứng dụng nào đang giữ tiêu điểm, miễn là máy khách desktop đang chạy (trên macOS, nhấn `Shift`+`Command`+`A`);
- Mọi màn hình tối đi dưới một lớp phủ chụp. Kéo để vẽ vùng chọn — nó có thể vươn qua vài màn hình — rồi kéo bên trong nó để di chuyển, hoặc kéo các tay cầm của nó để đổi kích thước; một huy hiệu trên vùng chọn cho biết kích thước tính bằng pixel;
- Nhấn `Esc` bất kỳ lúc nào để hủy.

![Lớp phủ chụp — vùng chọn với huy hiệu kích thước và thanh chú thích](images/terminal-images-capture.jpg)

## Chú thích

Thanh dưới vùng chọn mang bên trái các công cụ chú thích (hình chữ nhật và hình elip), ở giữa sáu màu, và bên phải ba nút hành động — hủy (`✕`), lưu (mũi tên tải về) và sao chép (`✓`).

- Chọn một công cụ và một màu, rồi kéo bên trong vùng chọn để vẽ; nét vẽ được cắt theo vùng chọn;
- `Ctrl`+`Z` hoàn tác chú thích cuối, `Ctrl`+`R` làm lại nó;
- Kéo một vùng mới ngoài vùng chọn hiện tại vứt bỏ nó — kể cả chú thích — và bắt đầu vùng khác.

## Sao chép hoặc lưu

- `Sao chép` (`✓`) đặt ảnh chụp đã chú thích vào clipboard — sẵn sàng dán vào terminal (dưới đây), một cuộc trò chuyện hoặc bất kỳ ứng dụng nào khác;
- `Lưu` (mũi tên tải về) ghi một PNG qua hộp thoại lưu; tên tệp mặc định là `vterm-screenshot-<timestamp>.png`.

## Dán ảnh vào terminal

- Sao chép một ảnh vào clipboard — từ công cụ chụp ở trên, một trình duyệt, hoặc tính năng chụp riêng của hệ điều hành;
- Chuột phải lên terminal và chọn `Paste Image`; mục này chỉ xuất hiện khi clipboard thực sự chứa ảnh;
- Ảnh được tải lên máy chủ của phiên và lưu ở đó thành tệp tạm; đường dẫn của nó trong dấu ngoặc kép sau đó được gõ vào vị trí con trỏ terminal đúng như bạn tự gõ — được bọc trong cờ bracketed paste khi ứng dụng đang chạy hỗ trợ, để shell, trình soạn thảo và tác tử nhận nó sạch sẽ;
- Với chương trình, nó chỉ là văn bản thuần: một tác tử mã như Claude Code hay opencode mở ảnh theo đường dẫn, và lệnh shell thường có thể dùng nó làm đối số.

![Chuột phải terminal — mục Paste Image](images/terminal-images-paste.jpg)

## Ghi chú

- Đường dẫn được chèn sống trên máy chủ của phiên: với máy chủ từ xa hoặc WSL, tệp tạm được tạo trên máy đó, đúng chỗ các chương trình chạy trong terminal với tới được;
- Kết hợp với `Thêm ngữ cảnh cho tác tử`, đây là cách nhanh nhất cho tác tử thấy thứ gì đó trên màn hình của bạn — chụp, dán, và tác tử nắm ảnh như một tệp;
- Trên Linux, tính năng chụp hoạt động dưới X11; phiên Wayland thuần không hỗ trợ cũng phím tắt toàn cục lẫn lớp phủ tương tác, và nếu một ứng dụng X11 khác đã giữ `Shift`+`Alt`+`A`, phím tắt không thể đăng ký.
