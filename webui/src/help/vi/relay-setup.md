Nối một máy vào relay mất hai trường: địa chỉ relay và khóa của nó. Xem trang trước, Tổng quan, để biết tính năng là gì và các mảnh ghép nhau ra sao.

## Thiết lập máy chủ relay

1. Mở mục `Server` trên máy sẽ đóng vai relay.
2. Chắc chắn `Secret Key` đã đặt — nút xúc xắc tạo một cái. Cùng khóa đó ghép cả máy khách lẫn máy chủ chia sẻ.
3. Mở rộng khối `QR Code` và chọn `Network Interface` / `IP Version` mà máy khách và máy chủ chia sẻ nên dùng để tiếp cận nó.

## Nối một máy chủ chia sẻ

Chi tiết kết nối relay di chuyển trên clipboard — không cần gõ lại:

1. Trong Web UI của máy chủ relay, nhấp `QR code`. Việc này sao chép một chuỗi chia sẻ được mã hóa (địa chỉ, cổng, khóa bí mật) vào clipboard.
2. Mở Web UI trên máy sắp chia sẻ, mục `Server`, và nhấp vào trường `Relay Address`.
3. Trình duyệt hiện hộp thoại hỏi có cho phép site này truy cập clipboard — nhấp `Allow`:

![Trình duyệt — hộp thoại quyền truy cập clipboard hiện khi trường được tập trung](images/relay-allow-paste.jpg)

4. Khi được cấp, clipboard được kiểm tra: khi chứa chuỗi chia sẻ, một nút dán hiện cạnh trường — nhấp để giải mã và tự điền `Relay Address` và `Relay Secret Key`:

![Web UI — địa chỉ relay và khóa bí mật relay tự điền từ chuỗi chia sẻ được dán](images/relay-paste-share.jpg)

5. Xem lại giá trị và nhấp `Apply` — lưu thì nối lại relay thượng nguồn.

![Web UI — các trường relay đã điền với nút Apply](images/relay-apply-share.jpg)

> Đọc clipboard cần bối cảnh an toàn (HTTPS hoặc localhost). Trên HTTP thuần nút dán giữ ẩn; hãy gõ tay địa chỉ và khóa relay.

Theo mặc định, kết nối thượng nguồn chỉ được mã hóa chứ không xác minh. Để đồng thời ghim chứng chỉ của relay, xem Xác minh nghiêm ngặt trên relay trong Chứng chỉ & TLS.

## Máy khách desktop

### Tab máy chủ chia sẻ

Tab của terminal chạy trên một máy chủ chia sẻ mang hai dấu: sọc màu ở mép trái tab — mỗi máy chủ một màu — và biểu tượng hệ điều hành từ xa (Windows / Apple / Linux) cạnh chấm trạng thái. Tab cục bộ trên chính máy chủ relay không có dấu.

![Máy khách desktop — dấu tab của terminal máy chủ chia sẻ](images/relay-tab-indicator.jpg)

### Hồ sơ terminal theo máy chủ

Menu hồ sơ (thanh tiêu đề) nhóm hồ sơ terminal theo máy chủ: trước hết hồ sơ của chính máy chủ relay đang nối, rồi một nhóm gập được mỗi máy chủ chia sẻ, đặt tên theo máy. Mở nhóm và chọn hồ sơ để mở terminal trên máy chủ đó. Menu theo nhóm tab được chọn trong thanh tiêu đề.

![Máy khách desktop — menu hồ sơ với nhóm theo máy chủ](images/relay-profile-menu.jpg)

### Nhóm và cửa sổ riêng

Bộ chọn nhóm thanh tiêu đề liệt kê nhóm tab của mọi máy chủ; nhóm mặc định của máy chủ chia sẻ hiển thị dạng `Ungrouped (Tên máy chủ)`. Menu con mỗi nhóm liệt kê tab của nó để chuyển nhanh và đề nghị `Move Group to New Window`, xé cả nhóm thành cửa sổ riêng — hữu ích để dành trọn cửa sổ hay màn hình cho một máy.

![Máy khách desktop — menu bộ chọn nhóm với hành động chuyển sang cửa sổ mới](images/relay-group-selector.jpg)
