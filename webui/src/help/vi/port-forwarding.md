Chuyển tiếp cổng xuất bản một dịch vụ HTTP đang chạy trên máy chủ — một dev server, một bảng điều khiển thiết bị, bất cứ thứ gì lắng nghe ở địa chỉ `http://localhost:port` thuần — trên cổng TLS riêng của nó, để bất kỳ thiết bị nào kết nối được tới máy chủ đều mở được nó trong trình duyệt dưới dạng `https://<địa-chỉ-máy-chủ>:<cổng>/`.

## Cách hoạt động

- Một chuyển tiếp ánh xạ một **cổng đích** — dịch vụ HTTP trên máy này — tới một **cổng được chuyển tiếp** được cấp từ dải cổng đã cấu hình (mặc định `62000`–`62999`);
- Cổng được chuyển tiếp dùng TLS: mọi lưu lượng đều được mã hóa bằng **chứng chỉ tự ký** của máy chủ — không gì truyền đi dạng văn bản thuần, kể cả qua mạng không đáng tin;
- Bất kỳ thiết bị nào kết nối được tới máy chủ — trong mạng LAN, hoặc từ internet khi dải cổng đã được chuyển tiếp trên router — mở `https://<địa-chỉ-máy-chủ>:<cổngĐượcChuyểnTiếp>/` và đến ngay dịch vụ cục bộ.

Chuyển tiếp được tạo theo nhu cầu: chia sẻ tệp `Host with HTTP server` trên máy chủ từ xa (qua relay) tự động yêu cầu một chuyển tiếp và trả về URL sẵn sàng để mở. Không cần thiết lập thủ công nào.

## Quản lý các chuyển tiếp

![Web UI — mục Port Forwarding: cấu hình dải cổng và danh sách các chuyển tiếp đang hoạt động](images/port-forwarding-webui.jpg)

1. Mở mục `Port Forwarding` của Web UI của máy chủ.
2. `Port Range Configuration` giới hạn nhóm mà các cổng được chuyển tiếp được cấp phát; thay đổi được lưu tự động. Thu hẹp nó khi tường lửa hoặc router của bạn chỉ chuyển tiếp một vài cổng.
3. Danh sách đang hoạt động hiển thị mỗi chuyển tiếp dưới dạng `:cổngĐượcChuyểnTiếp → :cổngĐích` kèm số kết nối đang hoạt động của nó.
4. Nút dừng của một chuyển tiếp gỡ listener ngay lập tức và ngắt các kết nối đang hoạt động của nó; lần chia sẻ tiếp theo sẽ được cấp một cổng mới. `Refresh` đọc lại danh sách.

## Cảnh báo chứng chỉ

Các chuyển tiếp được mã hóa bằng chính chứng chỉ tự ký của máy chủ — cùng chứng chỉ bảo vệ Web UI và các kết nối máy khách. Trình duyệt không thể xác minh nó với một tổ chức cấp chứng chỉ, nên lần đầu truy cập một URL được chuyển tiếp sẽ hiện cảnh báo riêng tư (*Kết nối của bạn không phải là kết nối riêng tư*). Điều này là bình thường và có thể an toàn bỏ qua: bản thân việc mã hóa không bị xâm phạm, chỉ là nguồn gốc của chứng chỉ không thể được máy xác minh. Để tắt hẳn cảnh báo này, hãy nhập chứng chỉ của máy chủ vào kho chứng chỉ tin cậy của thiết bị.

![Trình duyệt — cảnh báo chứng chỉ tự ký không đáng tin; tiếp tục qua Advanced → Proceed để mở trang](images/port-forwarding-cert-warning.jpg)

## Chrome trên macOS báo `ERR_ADDRESS_UNREACHABLE`

Trên macOS 15 (Sequoia) trở lên, ứng dụng bên thứ ba cần quyền rõ ràng để truy cập các thiết bị trong mạng cục bộ — bất kỳ địa chỉ nào dạng `192.168.x.x`, `10.x.x.x`, hay tương tự. Safari được miễn vì là ứng dụng hệ thống, nên một URL được chuyển tiếp có thể mở tốt trong Safari trong khi Chrome báo `ERR_ADDRESS_UNREACHABLE` trước cả khi TLS bắt đầu. Đây không phải vấn đề chứng chỉ và cũng không phải lỗi của bản thân chuyển tiếp.

Để cấp quyền cho Chrome:

1. Mở `Cài Đặt Hệ Thống` → `Quyền Riêng Tư & Bảo Mật` → `Mạng Cục Bộ`.
2. Tìm `Google Chrome` trong danh sách và **bật** nó lên.
3. Thoát hẳn Chrome (`Cmd+Q`, không phải chỉ đóng cửa sổ) rồi khởi chạy lại.

Nếu công tắc đã bật sẵn, hãy tắt rồi bật lại — nó đôi khi bị kẹt sau một bản cập nhật Chrome hoặc macOS. Nếu lỗi vẫn còn, hãy kiểm tra xem có tiện ích mở rộng VPN hay proxy nào trong Chrome không định tuyến các dải địa chỉ riêng hay không.

## Kiểm soát truy cập

Điều gì bảo vệ một cổng được chuyển tiếp phụ thuộc vào dịch vụ đứng sau nó:

- **Các chia sẻ tích hợp yêu cầu token.** Liên kết chia sẻ do máy khách tạo — chia sẻ `Host with HTTP server` (`?share_token=…`) và trang Settings (`?ticket=…`) — đều kèm token truy cập. Kết nối không có token hợp lệ và không có phiên hợp lệ sẽ bị cắt trước khi trả về bất kỳ nội dung nào. Ở lần truy cập đầu tiên, trình duyệt đổi token lấy cookie phiên bảo mật chỉ áp dụng cho dịch vụ đó, và token biến mất khỏi URL — không còn sót trong thanh địa chỉ hay lịch sử.

- **Token chỉ dùng một lần và sống rất ngắn.** Ticket của Settings chỉ có tác dụng đúng một lần và hết hạn sau một phút; khi dịch vụ chia sẻ dừng hoặc khởi động lại, các liên kết và phiên trước đó ngừng hoạt động. Hãy coi liên kết đầy đủ như một chứng chỉ — ai có nó là truy cập được dịch vụ — và chỉ chia sẻ cho những người bạn tin tưởng.

- **Dịch vụ riêng của bạn tự mang xác thực của mình.** Chuyển tiếp tới dịch vụ cục bộ khác chỉ mã hóa lưu lượng bằng chứng chỉ tự ký của máy chủ, không thêm bất kỳ lớp xác thực nào: mọi thứ dịch vụ đó phục vụ — cả trang lẫn API — đều nằm trong tầm với của bất kỳ ai kết nối được tới cổng. Hãy đặt xác thực cho các dịch vụ nhạy cảm trước khi xuất bản chúng.
