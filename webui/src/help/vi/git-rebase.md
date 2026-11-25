Rebase tương tác biến `git rebase -i` thành một dòng chảy trực quan: kéo để sắp xếp lại hàng đợi commit, áp commit từng cái một, giải xung đột trực quan — không cần dòng lệnh.

## Khởi động

Mở bảng nhánh và gõ vào ô `Filter branches...` để tìm nhánh đích, rồi chuột phải nó → `Rebase Current Branch onto '{branch}'` để rebase nhánh hiện tại lên đích. Bảng chuyển sang chế độ xem rebase, đi qua bốn pha:

![Tìm nhánh trong bảng, rồi khởi động rebase lên đích](images/git-rebase-search.jpg)

## 1. Chỉnh hàng đợi commit

Các commit trên nhánh của bạn mà đích thiếu tạo thành hàng đợi:

- **Kéo** để sắp xếp lại;
- mỗi commit có menu hành động: `pick` (giữ) / `squash` (gập vào cái trước, gộp thông điệp) / `fixup` (gập, bỏ thông điệp) / `edit` (dừng để sửa) / `drop` (bỏ);
- commit bị squash và drop gập lại trực quan.

![Hàng đợi commit rebase — sắp xếp lại bằng kéo, hành động mỗi commit](images/git-rebase-queue.jpg)

## 2. Áp

`Start Rebase` phát lại hàng đợi lên đích từng commit một với thanh tiến độ trực tiếp; không xung đột thì chạy thẳng tới xong. `Abort` thoát ra bất cứ lúc nào.

## 3. Xung đột (nếu có)

Xung đột khi áp chuyển sang chế độ xem xung đột — danh sách tệp cộng trình soạn thảo gộp bốn chế độ của nó (`Merge` / `Split` / `Raw` / `AI`), nơi mỗi hunk xung đột nhận quyết định riêng (`Ours` / `Theirs` / `Both` …; xem Giải quyết xung đột). Khi mọi tệp được giải, `Continue` tiếp tục rebase.

![Chế độ xem xung đột — quyết định gộp từng hunk](images/git-rebase-resolve-conflict.jpg)

## 4. Xong

Khi hoàn tất bảng xác nhận `Branch '{branch}' was rebased onto '{target}'` và hiện thẻ push — rebase đã viết lại lịch sử nên nhánh từ xa phải cập nhật cưỡng bức:

- `Force Push with Lease` — khuyến nghị: chỉ ghi đè khi không ai khác push và không bao giờ dùng `--force` trơn;
- `Not now` — bỏ qua push; rebase cục bộ đã hoàn tất.

![Trang hoàn tất — Force Push with Lease hoặc bỏ qua push](images/git-rebase-done.jpg)

> Phiên vẫn dùng được trong lúc rebase; đóng bảng không gián đoạn cái đang chạy.
