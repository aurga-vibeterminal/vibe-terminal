Biến môi trường cho phép bạn duy trì các bộ biến theo nhóm mỗi dự án (`ANDROID_HOME`, khóa API, cài đặt proxy) và áp hoặc xóa chúng trên bất kỳ phiên terminal nào bằng một cú nhấp. Chúng sống trong cấu hình máy chủ; giá trị được mã hóa khi lưu.

## Duy trì trong Web UI

Mục `Environment Variables` tổ chức chúng thành cây:

- **Nhóm** — thùng chứa thuần, lồng được (chuột phải → `New Group`), để tổ chức theo dự án hay kịch bản;
- **Bộ sưu tập** — lá của cây và nơi thực sự giữ biến (`New Leaf`); chỉ bộ sưu tập mới áp được lên terminal;
- cặp khóa/giá trị của mỗi bộ sưu tập được sửa qua `Edit Variables` (`Add Variable`); giá trị mặc định ẩn;
- bộ sưu tập có thể nhân bản để tạo biến thể (cùng bộ với một khóa đổi).

![Trang Environment Variables — cây nhóm/bộ sưu tập](images/terminal-env-tree.jpg)

## Áp lên terminal

Chuột phải trong terminal → `Envs`:

- menu con phản chiếu cây từ Web UI; mở rộng một bộ sưu tập và chọn `Apply` để tiêm biến của nó vào phiên hiện tại, hoặc `Clear` để gỡ lại;
- tiến trình mới khởi động thấy biến (chương trình tiền cảnh đang chạy thì không);
- bộ sưu tập hiện gấp dưới nhóm của chúng; cả mục `Envs` ẩn khi cây không có bộ sưu tập.

![Chuột phải terminal — áp và xóa trong menu con Envs](images/terminal-env-menu.mp4)
