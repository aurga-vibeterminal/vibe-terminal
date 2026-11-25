Có hai cách dùng WSL trên Windows: cách nhẹ thêm một shell WSL làm hồ sơ terminal, cách đầy đủ cài vterm-server bên trong WSL. Cách trước chỉ cho bạn shell; cách sau mở khóa quản lý tệp, bảng Git, MCP và mọi thứ khác.

## Cách nhẹ: hồ sơ terminal WSL

Trên Windows, **hồ sơ terminal WSL tự động được nạp**: lúc khởi động máy chủ phát hiện `wsl.exe` và liệt kê các bản phân phối đã cài, thêm một mục hồ sơ cho mỗi distro vào danh sách (vd. `Ubuntu (WSL)`; một mục chung `WSL` hiện khi không có distro nào).

Để dùng, chỉ cần chọn mục WSL trong menu hồ sơ thanh tiêu đề của máy khách — distro khởi chạy và thả bạn thẳng vào terminal của nó, như mở một cái cục bộ.

Hạn chế: các phiên đó chạy trên máy chủ phía Windows — **trình khám phá tệp, bảng Git, biến môi trường v.v. của máy khách không hoạt động** (chúng vận hành theo ngữ nghĩa hệ tệp Windows và không nhìn thấy hệ tệp Linux trong WSL).

![Menu hồ sơ terminal — các distro WSL liệt kê tự động](images/terminal-wsl-profile.jpg)

## Cách đầy đủ: cài vterm-server trong WSL

Coi WSL như một máy Linux riêng và cài vterm-server headless. Trang web cung cấp script một dòng — dán nó vào terminal WSL:

```bash
curl -fsSL https://vterminal.dev/install.sh | bash
```

Script tự động:

1. Phát hiện môi trường WSL/headless và tải vterm-server **headless** vào `~/.local/vterm`;
2. Cài một **dịch vụ người dùng systemd** và khởi động ngay — nó chạy mỗi lần đăng nhập (kể cả khởi động WSL), không cần khởi động thủ công;
3. Thêm `~/.local/vterm` vào PATH của bạn.

> Khi systemd trong WSL bị tắt, script chỉ cho bạn thêm `[boot]` `systemd=true` vào `/etc/wsl.conf`, chạy `wsl --shutdown` phía Windows và chạy lại trình cài đặt.

## Mở trang cấu hình

Trong WSL:

```bash
vterm-server --open
```

Lệnh in địa chỉ Web UI (`http://localhost:<config-port>/`) và cố mở trình duyệt (trong WSL qua `wslview`, trình duyệt mặc định của Windows). Nhờ chuyển tiếp localhost của WSL2, **mở cùng địa chỉ trong trình duyệt Windows** cũng tốt như nhau. Khi máy chủ đang chạy, URL in ra kèm token đăng nhập dùng một lần (`?ticket=...`, cấp qua socket cục bộ) nên trang mở ra đã được xác thực.

## (Tùy chọn) Chia sẻ máy chủ WSL vào relay của máy chủ lưu

Nếu máy Windows cũng chạy vterm-server (đóng vai relay), máy chủ WSL có thể gắn dưới nó để máy khách thấy cả hai "máy" qua một kết nối cục bộ:

1. Trong Web UI của máy chủ Windows, nhấp mã QR để sao chép chuỗi chia sẻ;
2. Mở Web UI của máy chủ WSL (địa chỉ từ phần trước), nhấn nút dán hiện cạnh trường `Relay Address` để tự điền địa chỉ và khóa, rồi `Apply`;
3. Chừng nào vterm-server Windows còn chạy, máy khách thấy terminal, tệp và Git của máy chủ WSL dưới relay cục bộ.

> Dưới mạng NAT mặc định của WSL2, WSL tới máy chủ Windows qua địa chỉ máy chủ nhìn từ phía WSL (cổng mặc định — nameserver trong `cat /etc/resolv.conf`); với mạng đối chiếu `localhost` hoạt động. Xem chương Relay để biết chi tiết relay.
