Khi máy chủ đã cấu hình xong, mọi bề mặt của client đều cung cấp dịch qua cùng một biểu tượng quả cầu. Kết quả được lưu đệm theo từng mục, nên đổi qua lại không bao giờ lặp lại yêu cầu.

## Tin nhắn trong bảng trò chuyện

Phần đầu mỗi tin nhắn đều có biểu tượng dịch — cạnh thời gian trên tin của bạn, bên phải tên người gửi và thời gian trên phản hồi của mô hình, và bên phải nhãn `Đang suy nghĩ...` trên các khối suy luận:

- **Cú nhấp đầu** gửi Markdown thô của tin nhắn tới máy chủ và hoán đổi nội dung tin nhắn lấy bản dịch. Biểu tượng sáng lên khi bản dịch đang hiển thị;
- **Nhấp lần nữa** khôi phục văn bản gốc;
- Bản dịch bám theo tin nhắn chừng nào cuộc hội thoại còn mở, kể cả các khối `Đang suy nghĩ...` sau khi mở rộng.

Tin nhắn đã dịch hiển thị như Markdown bình thường, nên danh sách, bảng và khối mã giữ nguyên hình dáng.

![Một tin nhắn trò chuyện với biểu tượng dịch ở phần đầu và nội dung đã dịch bên dưới](images/translation-chat-message.jpg)

## Bản nháp thông điệp commit

Trong ô commit của bảng Git, một nút dịch xuất hiện cạnh nút AI ngay khi bản nháp vượt quá một dòng:

- **Nhấp** để dịch toàn bộ bản nháp — văn bản đã dịch thay thế bản nháp trong ô dưới dạng bản xem trước chỉ đọc;
- **Nhấp lại** để quay về câu chữ gốc của bạn (bản nháp thực tế chưa bao giờ bị sửa);
- Không gõ được trong lúc bản xem trước hiển thị; đổi về trước rồi mới tiếp tục chỉnh sửa.

![Ô commit với nút dịch và bản xem trước đã dịch](images/translation-commit-draft.jpg)

## Cửa sổ chi tiết commit

Rê chuột lên một commit trong lịch sử Git để mở cửa sổ chi tiết. Cạnh nút sao chép mã băm là biểu tượng dịch: nhấp để dịch thông điệp commit trong cửa sổ; nhấp lần nữa để xem bản gốc. Thông điệp đã dịch hiển thị như bản gốc — dòng tiêu đề, nội dung Markdown và khối mã đều đủ.

![Cửa sổ chi tiết commit — nút sao chép và dịch trên dòng mã băm](images/translation-git-tip.jpg)

## Vùng chọn trong trình soạn thảo mã

Chọn văn bản trong trình soạn thảo mã — vùng chọn thường và khối (hình chữ nhật) đều được — nhấp chuột phải và chọn `Dịch` (mục đầu tiên của menu):

1. Một huy hiệu quả cầu đặc hiện trên số dòng của dòng chọn đầu tiên;
2. Thẻ bản dịch mở ngay với chỗ trống *Translating…* và lấp đầy khi máy chủ phản hồi;
3. Thẻ này bám chặt: nó đứng yên khi bạn cuộn, và chỉ nút đóng, `Esc` hay nhấp ra ngoài mới gỡ bỏ nó. Rê lên huy hiệu sẽ mở lại.

Mỗi vùng chọn đã dịch có huy hiệu riêng, và dịch lại cùng những dòng đó chỉ đơn giản hiện lại thẻ đã lưu. Huy hiệu biến mất khi bạn mở tệp khác.

![Vùng chọn trong trình soạn thảo mã với Dịch trong menu ngữ cảnh](images/translation-editor-menu.jpg)

## Vùng chọn trong xem diff

Chế độ xem diff hoạt động y hệt: chọn các dòng trong thân diff, chuột phải, `Dịch`. Huy hiệu thay số dòng mới của dòng chọn đầu tiên, và thẻ kết quả neo ngay bên dưới. Vùng chọn ở chế độ chỉnh sửa cũng dịch được; chuyển sang tệp hay commit khác sẽ xóa các huy hiệu.

![Một diff với vùng chọn đã dịch — huy hiệu rãnh số dòng và thẻ bản dịch](images/translation-diff-card.jpg)

## Cái gì được dịch

Máy chủ cho mô hình biết văn bản đến từ đâu:

- **Văn xuôi** (tin nhắn trò chuyện, thông điệp commit) — định dạng Markdown được giữ nguyên và chỉ ngôn ngữ tự nhiên được dịch;
- **Vùng chọn từ tệp** (trình soạn thảo, diff) — tên tệp đi cùng yêu cầu, nên mô hình biết các ký hiệu chú thích của tệp (`//`, `#`, `--`, …). Ký hiệu chú thích được lược bỏ và văn bản chú thích được dịch như văn xuôi; Markdown bên trong chú thích (tiêu đề, danh sách, khối mã) tồn tại. Nếu vùng chọn là mã thường, chỉ chú thích và hằng chuỗi được dịch — định danh và cấu trúc giữ nguyên vẹn.
