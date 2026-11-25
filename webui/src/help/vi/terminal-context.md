Các tác tử mã (Claude Code, Codex, Kimi, Pi, OpenCode…) chỉ biết những gì tới đầu vào của chúng. `Thêm ngữ cảnh` là cách trao đường dẫn tệp và đoạn mã mà không phải gõ lại: kéo tệp từ `Explorer`, hoặc kéo văn bản được chọn từ trình soạn thảo mã và khung diff Git, thẳng vào terminal của tác tử hay ô nhập trò chuyện Copilot.

## Kéo tệp vào

Nhấn một tệp — hoặc một chọn nhiều — trong danh sách tệp `Explorer` và kéo nó:

- **Lên một terminal**: các đường dẫn đầy đủ được gõ tại điểm thả, phân tách bằng dấu cách, sẵn sàng thành một phần lời nhắc tiếp theo của bạn cho tác tử;
- **Lên ô nhập trò chuyện Copilot**: các đường dẫn được chèn vào bản thảo thông điệp, mỗi dòng một cái, để bạn lắp một lời nhắc nhiều tệp trước khi gửi.

Các đường dẫn kéo từ trình quản lý tệp của hệ điều hành và thả lên terminal được tự động trích dẫn, nên tên chứa dấu cách sống sót nguyên vẹn.

![Kéo tệp từ danh sách Explorer vào terminal gõ đường dẫn đầy đủ của nó](images/terminal-context-drag-file.gif)

## Kéo mã được chọn làm ngữ cảnh

Chọn văn bản trong trình soạn thảo mã hoặc trong diff Git, rồi nhấn chính vùng chọn và kéo nó lên terminal:

- khi terminal **đang chạy tác tử mã**, việc thả chèn một *khối ngữ cảnh* — đường dẫn tệp, khoảng dòng, và đoạn mã bọc trong hàng rào mã:

````text
File: crates/vterm/src/ui/terminal_view.rs
Lines: 558-566
```
fn can_accept_native_drop(&self, info: &DndInfo) -> bool { … }
```
````

- khi terminal ở **dấu nhắc shell thuần**, việc thả gõ văn bản thô được chọn thay vào đó, nên cùng cử chỉ này cũng dùng cho lệnh thường;
- khi diff được xem tại một commit cụ thể, khối còn kèm dòng `Commit:` ghim đúng bản sửa đổi.

![Kéo vùng chọn trình soạn thảo vào terminal tác tử thả khối ngữ cảnh File/Lines](images/terminal-context-drag-selection.mp4)

## Sao chép ngữ cảnh

Cùng một khối nằm trong menu chuột phải: `Copy Context` trong trình soạn thảo mã và khung diff sao chép nó vào clipboard. Dán vào ô nhập trò chuyện Copilot — hoặc bất kỳ tác tử nào khác — khi kéo bất tiện. Phím hoạt hình trên cho thấy khối thu được trong ô nhập của tác tử.

## Mẹo

- Chọn nhiều trong `Explorer` để thả vài đường dẫn cùng lúc;
- ô nhập trò chuyện Copilot gửi thẳng vào terminal như văn bản gõ (xem `Lịch sử tác tử mã`), nên ngữ cảnh thêm ở đó tới tác tử y như bạn tự gõ;
- kết hợp với `Tự động xác nhận` để tác tử hành động trên ngữ cảnh được thả mà không cần lời nhắc thêm.
