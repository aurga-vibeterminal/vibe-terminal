Vùng `CHANGES` của bảng Git bao phủ vòng lặp commit hằng ngày: xem lại thay đổi, staging, hủy, stash và commit kèm thông điệp.

## Stage và unstage

- Các thay đổi chưa stage được liệt kê dưới `CHANGES`, mỗi tệp mang một huy hiệu trạng thái — `M` sửa, `A` thêm, `D` xóa, `?` không theo dõi, `!` xung đột (hàng submodule mang thêm thẻ `SUB`);
- rê trên một hàng lộ các nút hành động nội tuyến: `+` stage một tệp chưa stage, `−` unstage một tệp đã stage, và `×` trên hàng chưa stage hủy nó; các nút `Stage All Changes` / `Unstage All Changes` trên đầu mục xử lý tất cả một lần;
- `Ctrl`-nhấp chuyển tệp vào chọn nhiều và `Shift`-nhấp chọn một dải — một nút hành động khi đó áp cho mọi tệp được chọn;
- nhấp tên tệp mở diff của nó để bạn xem lại thay đổi trước khi quyết định;
- chuột phải một tệp đề nghị `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` và lối tắt `cd` nhảy terminal tới thư mục tệp.

![Mục STAGED CHANGES — tệp đã stage với nút Unstage mỗi hàng](images/git-staging-staged.jpg)

## Commit

1. Stage những gì nên vào;
2. gõ thông điệp vào hộp commit (`Ctrl+Enter` commit) hoặc nhấn nút AI để tạo một cái (xem Trợ giúp LLM);
3. nhấp `Commit`.

![Hộp commit — ô thông điệp với nút tạo bằng AI](images/git-staging-commit.jpg)

Sau khi commit, rê lên đầu lịch sử — thanh mang tên nhánh phía trên lịch sử commit — và các nút `Fetch`, `Pull` và `Push` hiện dần bên phải. `Push` gửi ngay nhánh hiện tại tới upstream; cho các tùy chọn như `Force with lease`, dùng hộp thoại push của danh sách nhánh (xem Push, checkout và khác).

![Rê lên đầu lịch sử lộ các nút Fetch, Pull và Push](images/git-staging-push.jpg)

## Stash

- Nút lưu trữ trên đầu `CHANGES` mở hộp thoại `Stash All Changes`, nơi một thông điệp tùy chọn mô tả stash;
- chuột phải các tệp chưa stage được chọn đề nghị `Stash This File` (hoặc `Stash <n> Selected Files` cho chọn nhiều) để stash đúng các đường dẫn đó — hàng submodule bị loại;
- stash gom dưới `STASHES` thành các mục `WIP on <nhánh>`; bung một cái liệt kê các tệp nó bắt, và nhấp một tệp hiện diff của nó;
- rê trên hàng stash hiện `Apply` (khôi phục và giữ), `Pop` (áp và gỡ) và `Drop` (gỡ không áp) — `Pop` và `Drop` hỏi xác nhận trước, và menu chuột phải thêm `View Diff`;
- mục `STASHES` giữ ẩn khi danh sách stash trống.

![Hộp thoại Stash All Changes với thông điệp tùy chọn](images/git-staging-stash-all.jpg)

![Mục STASHES — một stash bung ra các tệp, với nút Apply, Pop và Drop](images/git-staging-stashes.jpg)

![Pop áp stash và gỡ nó khỏi danh sách sau xác nhận](images/git-staging-stash-pop.jpg)

## Hủy

`Discard Changes` trên một tệp chưa stage vứt mọi thay đổi chưa commit trong nó — có hộp thoại xác nhận và hành động **không thể hoàn tác**.

> Một commit chỉ gồm nội dung đã stage; thay đổi chưa stage ở lại cây làm việc.
