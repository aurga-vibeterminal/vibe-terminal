Relay giải một vấn đề: phần lớn máy nằm sau NAT hay IP động nên máy khách không thể quay thẳng tới từng máy. Hãy để một máy tiếp cận được công khai đóng vai điểm vào relay; mọi máy khác quay ra nó. Một máy khách khi đó duy trì một kết nối duy nhất tới relay và qua đó tới terminal của mọi máy chủ chia sẻ.

## Kiến trúc

![Kiến trúc relay — mỗi máy khách giữ một kết nối tới relay; máy chủ chia sẻ quay lên relay](diagram/relay)

- **Máy chủ relay** — điểm vào công khai mà máy khách và máy chủ chia sẻ kết nối tới. Để trống `Relay Address` của nó.
- **Máy chủ chia sẻ** — một máy mà `Relay Address` của nó trỏ về relay. Nó quay lên và chia sẻ terminal của mình qua relay. Sọc màu trên mỗi nút chia sẻ trong sơ đồ phản chiếu sọc mà máy khách desktop vẽ trên tab của máy chủ đó — mỗi máy chủ một màu.
- Kết nối luôn khởi phát từ bên trong: máy chủ chia sẻ quay relay nên chúng không cần chuyển tiếp cổng hay IP công khai.

## Máy khách thấy gì

Khi đã nối relay, mọi máy chủ chia sẻ ứng xử như một máy cục bộ:

- Tab terminal mang sọc màu và biểu tượng hệ điều hành từ xa, nhận diện máy trong một cái liếc (xem Hướng dẫn thiết lập);
- menu hồ sơ nhóm terminal theo máy chủ — mở terminal trên bất kỳ máy nào trực tiếp;
- nhóm tab có thể xé thành cửa sổ riêng, dành trọn cửa sổ cho một máy.

## Mô hình bảo mật

Máy khách và máy chủ chia sẻ ghép nối bằng cùng một khóa bí mật; kết nối được hầm qua relay. Khóa di chuyển trong mã QR ghép nối / chuỗi chia sẻ, không bao giờ dạng văn bản thuần.

Hướng dẫn từng bước ở trang kế, Hướng dẫn thiết lập.
