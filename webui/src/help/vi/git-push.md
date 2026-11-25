Menu chuột phải của danh sách nhánh tập trung push, checkout và các thao tác nhánh hằng ngày.

## Push

Chuột phải một nhánh → `Push '{branch}'...` mở hộp thoại push:

![Chuột phải nhánh — mục Push](images/git-push-contextmenu.jpg)

- Nó hiện ánh xạ đích `local → remote`; cả hai tên đều đổi được (push lên nhánh từ xa sẵn có với tên khác sẽ xin xác nhận);
- Tùy chọn: `Push all branches`, `Set upstream` (buộc upstream ở lần push đầu), `Push tags`, `Dry run`;
- `Force` viết lại lịch sử và `Force with lease` an toàn hơn (từ chối khi người khác đã push — được chuộng). Các thao tác viết lại lịch sử, như push sau rebase, cần một trong hai;
- một gợi ý rủi ro về force push được hiển thị trong hộp thoại.

![Hộp thoại push — đích local → remote và các hộp kiểm tùy chọn](images/git-push-dialog.jpg)

## Checkout

Chuột phải một nhánh → `Checkout` để đổi nhánh — nhánh cục bộ checkout thẳng, còn nhánh từ xa hay tag mở hộp thoại checkout:

![Chuột phải nhánh — mục Checkout](images/git-checkout-contextmenu.jpg)

- `New branch name` — nhánh cục bộ cần tạo, điền sẵn từ ref;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, chỉ nhánh từ xa), `Force` (`-f`), `Merge` (`--merge`).

![Hộp thoại checkout — tên nhánh và tùy chọn](images/git-checkout-dialog.jpg)

Thư mục làm việc của phiên theo sự chuyển đổi; terminal và bảng Git phản ánh nhánh mới ngay.

## Detached HEAD

Checkout một commit trần (vd. checkout một commit khi duyệt lịch sử) tách HEAD. Commit kế tiếp khi đó mở hộp thoại `Detached HEAD`: chọn một nhánh sẵn có để commit, `+ Create new branch…` tại chỗ, hoặc tạm `Skip`.

## Thao tác khác

- **Stash** — stash mọi thay đổi lên đống (thông điệp tùy chọn) từ đầu thay đổi; danh sách `STASHES` hỗ trợ `Pop` (áp và gỡ) và `Drop` (bỏ, không thể hoàn tác);
- **Xóa nhánh/worktree** — qua menu chuột phải của mục, cả hai kèm xác nhận hành động không thể hoàn tác;
- **Lịch sử & khôi phục** — `HISTORY` liệt kê commit; chuột phải cho lịch sử tệp, diff với cây làm việc, hoặc khôi phục về một bản sửa đổi;
- **Sao chép** — tên nhánh và băm commit sao chép một cú nhấp, tiện dán cho tác tử hay đồng nghiệp.
