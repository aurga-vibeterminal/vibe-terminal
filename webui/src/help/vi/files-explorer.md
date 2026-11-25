Trình khám phá tệp vận hành hệ tệp phía máy chủ. Thanh công cụ trên đầu bảng trái chuyển giữa Recent / Explorer / Git / Debug / Search; Explorer là trình duyệt tệp.

## Nút Home: nhảy tới cwd của terminal

Nhấn nút `Home` (hình nhà) trên thanh công cụ:

- Chuyển sang chế độ xem `Explorer`;
- Nhảy tới **thư mục làm việc của terminal đang hoạt động** (cwd của nó);
- Ghim thư mục đó thành cây và xóa ô tìm kiếm — bạn luôn bắt đầu từ "terminal ở đâu, thấy ở đó".

![Trình khám phá tệp ở chế độ thường — Home, breadcrumb và danh sách phẳng](images/files-explorer.jpg)

## Chế độ cây và tìm kiếm

- **Ghim thành cây**: nút ghim ở cuối phải của breadcrumb chuyển giữa danh sách phẳng và chế độ cây. Ở chế độ cây các thư mục bung thành cây thư mục gập được (cú nhảy Home tự ghim);
- **Tìm kiếm**: ô tìm kiếm phía trên lọc danh sách hiện tại theo tên khi bạn gõ; `Search in Folder` từ menu ngữ cảnh của Recent chạy tìm kiếm đệ quy trên cả thư mục.

![Trình khám phá tệp ở chế độ cây — gốc được ghim bung thành cây thư mục gập được](images/files-explorer-tree.jpg)

## Thao tác cơ bản

Chuột phải một tệp hay thư mục:

- **Tạo**: `New Folder` / `New File`;
- **Sửa**: `Rename`; `Copy` / `Cut` / `Paste` (hoạt động xuyên thư mục);
- **Xóa**: `Delete`, kèm hộp thoại xác nhận;
- **Liên kết terminal**: `Open in Terminal` mở tab terminal mới tại thư mục đó; `cd to Directory` cd terminal hiện tại tới đó;
- **Liên kết hệ điều hành**: `Reveal in File Explorer` hiện một thư mục cục bộ trong trình quản lý tệp hệ thống;
- **Đường dẫn**: `Copy Full Path` / `Copy Relative Path`;
- **Xuất bản**: `Host with HTTP server` xuất bản thư mục làm dịch vụ tệp HTTP (xem Xuất bản bằng máy chủ HTTP); `Open in App` mở nó bằng ứng dụng tác tử được phát hiện trên máy chủ.

Mọi thao tác tác động lên hệ tệp phía máy chủ; phiên từ xa ứng xử y như cục bộ.
