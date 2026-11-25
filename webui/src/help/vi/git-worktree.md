Worktree checkout nhiều nhánh của một kho sang các thư mục riêng: phát triển song song mà không đụng cây làm việc chính, rồi gộp nhánh lại và dọn dẹp. Vibe Terminal biến cả chu trình thành trỏ-và-nhấp.

## Tạo worktree

1. Trong danh sách nhánh của bảng Git, chuột phải nhánh đích (cục bộ hoặc từ xa) → `New Worktree`;
2. hộp thoại đặt tên worktree mới theo nhánh — sửa nếu thích; xác nhận, và máy chủ tạo thư mục worktree và checkout nhánh ở đó;
3. **một tab terminal mới tự mở** với thư mục làm việc đặt vào worktree mới — bắt đầu phát triển biệt lập ngay khi cây làm việc chính còn ở nhánh của nó.

![Hộp thoại New Worktree — tên mặc định là nhánh](images/git-worktree-create.jpg)

![Tab mới mở cho worktree vừa tạo](images/git-worktree-terminal.jpg)

## Làm việc song song

Mỗi worktree là một thư mục làm việc hoàn toàn độc lập: build và kiểm thử cho nhánh chính và nhánh tính năng chạy song song trong các tab khác không can nhiễu. Nhánh được checkout trong worktree mang một điểm ghi worktree hiển thị đường dẫn của nó trong danh sách nhánh, và commit lẫn push chạy ngay từ tab worktree như trên bất kỳ nhánh nào.

Chuột phải một nhánh worktree thêm hai mục riêng worktree: `Reveal in File Explorer` (phiên cục bộ) mở thư mục của nó trong trình quản lý tệp hệ thống, và `Open in Terminal` mở thêm một tab ở đó.

## Gộp lại và dọn dẹp

1. Commit và push trong tab worktree như thường;
2. chuyển tab chính sang nhánh đích (vd. `main`), chuột phải nhánh tính năng → `Merge into Current Branch` để đưa công việc qua;

   ![Gộp nhánh worktree trở lại nhánh chính](images/git-worktree-merge.jpg)

3. khi đã gộp, chuột phải nhánh → `Delete Worktree`: việc này gỡ thư mục worktree và nhánh của nó cùng nhau. Hãy chắc mọi thứ đã gộp hoặc push trước — hộp thoại cảnh báo lần nữa và hành động không thể hoàn tác.

   ![Xác nhận Delete Worktree — thư mục và nhánh bị gỡ cùng nhau](images/git-worktree-delete.jpg)

