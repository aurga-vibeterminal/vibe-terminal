Khi rebase hay gộp gặp xung đột, bảng chuyển sang chế độ xem xung đột: danh sách tệp xung đột bên trái, trình soạn thảo gộp bên phải. Tệp xung đột cũng hiện trong danh sách `CHANGES` với huy hiệu `!` — nhấp một cái nhảy thẳng vào chế độ xem này thay vì diff trơn. Trình soạn thảo đề nghị bốn chế độ giải và bạn có thể chuyển giữa chúng bất cứ lúc nào.

![Chế độ xem xung đột — danh sách tệp trái, trình soạn thảo gộp phải](images/git-conflicts-view.jpg)

## Danh sách tệp xung đột

Tệp được nhóm theo trạng thái: những cái tự gộp thành công (tự stage) và những cái còn cần chú ý tay. Nhấp một tệp để mở trong trình soạn thảo; danh sách cập nhật trực tiếp khi bạn giải, cho tới khi rỗng.

## Bốn chế độ giải

- **Merge** (mặc định) — xung đột được trình bày từng hunk; mỗi hunk chọn `take ours` / `take theirs` hoặc sửa kết quả gộp nội tuyến;
- **Split** — một diff được bẻ thành các đoạn nhỏ cấp dòng, nên hai bên có thể tiếp nhận từng mảnh trong một hunk;
- **Raw** — chỉnh sửa trơn với các dấu xung đột `diff3` (`<<<<<<<` / `=======` / `>>>>>>>`) để tinh chỉnh tay;
- **AI** — một tác tử giải tệp từng hunk: LLM phân tích mỗi hunk xung đột và tạo thẻ giải pháp kèm giải thích và độ tin cậy. Bạn có thể gõ chỉ dẫn để dẫn dắt chiến lược (vd. "ưu tiên theirs, giữ các câu lệnh log"), tạm dừng, tinh chỉnh ở chế độ khác, rồi tiếp — hunk đã giải được bỏ qua.

Ở mọi chế độ, lời giải được ghi vào tệp trong phiên của bạn và stage, y như chỉnh tay.

## Tiếp tục rebase

Khi mọi xung đột được giải, rebase tiến sang commit kế; bạn cũng có thể hủy bất cứ lúc nào để trở về trạng thái trước rebase.
