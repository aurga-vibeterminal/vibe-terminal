Bảng Git đưa quản lý phiên bản vào máy khách: staging, commit, quản lý nhánh, worktree, rebase tương tác, giải quyết xung đột và push — mà không rời cửa sổ terminal. Mọi thứ vận hành trên kho Git của phiên terminal hiện tại: phiên chạy trong thư mục nào thì đó là kho bảng hiển thị.

## Mở nó

Nhấp tab `Git` trong thanh hoạt động trái (dữ liệu kho chỉ xuất hiện khi phiên đang hoạt động nằm trong một kho Git). Kho của tab theo phiên hoạt động của nó; breadcrumb kho cho phép nhảy giữa các kho cha-con, và submodule được liệt kê riêng.

![Bảng Git — thay đổi, lịch sử và nhánh](images/git-overview-panel.jpg)

## Nó làm được gì

- **Staging & commit** — stage/unstage theo tệp hoặc hàng loạt, hủy thay đổi, commit kèm thông điệp, thông điệp commit do AI tạo (xem Trợ giúp LLM);
- **Lịch sử** — lịch sử commit và theo tệp, so sánh và khôi phục giữa các bản sửa đổi;
- **Nhánh** — checkout, tạo và xóa nhánh cục bộ/từ xa và tag;
- **Worktree** — checkout một nhánh sang cây làm việc biệt lập (một tab terminal mới tự mở), gộp lại khi xong (xem Worktree);
- **Rebase tương tác** — sắp xếp lại/squash/chỉnh hàng đợi commit trực quan kèm giải quyết xung đột (xem Rebase tương tác);
- **Giải quyết xung đột** — một trình soạn thảo gộp bốn chế độ, gồm tác tử AI giải từng hunk một (xem Giải quyết xung đột);
- **Push & linh tinh** — hộp thoại push đầy đủ, stash, xử lý detached-HEAD (xem Push, checkout và khác).

Các trang sau bao phủ từng vùng theo thứ tự đó.
