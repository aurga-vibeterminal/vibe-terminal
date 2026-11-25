Có hai mức tùy chỉnh: vá mã nguồn của addon hiện có, hoặc viết addon mới theo mẫu addon tích hợp rồi tải lên.

## Vá nguồn kiểu nóng

Nút bút chì trên bất kỳ thẻ addon nào mở mã JavaScript để chỉnh. Lưu sẽ cất một bản chụp ghi đè bản gốc — với cả addon tích hợp lẫn đã tải lên — và thay đổi có hiệu lực ngay, không cần khởi động lại. Hữu ích để chỉnh prompt, sửa URL, hoặc thử ý tưởng trên hệ thống đang chạy.

`Reset` bỏ bản chụp và khôi phục mã gốc (chỉ addon tích hợp — addon đã tải lên vốn là của bạn).

![Chỉnh mã nguồn addon — bản chụp đã lưu ghi đè bản gốc](images/addon-edit-source.jpg)

## Viết addon của riêng bạn

Một addon gồm hai tệp:

- `<id>.json` — manifest: `id`, `name`, `description`, `author`, `email`, `date`, `type`, và `config_fields` (key, label, kiểu trường, mặc định, mô tả) mà Web UI biến thành biểu mẫu;
- `<id>.js` — mã nguồn: tệp JavaScript thuần (chạy bởi engine QuickJS nhúng, với cầu `addon.get_config()` tới các trường đã cấu hình), xuất các hàm mà loại của nó gọi — vd. `chat_completion()` cho nhà cung cấp LLM, danh sách công cụ và lời gọi cầu cho addon MCP.

Cách bắt đầu nhanh nhất là xuất một addon sẵn có: nút `Export` trên thẻ của nó tải về một zip chứa manifest và mã nguồn cùng nhau. Chọn addon tích hợp gần mục tiêu nhất (chúng cũng nằm trong `crates/vterm-server/src/addon/builtins/`), điều chỉnh id, name, type và các hàm xuất, rồi cài kết quả bằng `Upload Addon`. Addon đã tải lên có thể xóa lại bằng nút thùng rác.

![Xuất addon — manifest và mã nguồn JavaScript trong một zip](images/addon-export.jpg)

## Lưu trữ

Addon đã tải lên và các bản chụp mã nguồn nằm trong thư mục `addons/` của thư mục cấu hình dưới dạng `<id>.json` + `<id>.js`, nên tồn tại qua các lần khởi động lại. Với id tích hợp, chỉ phần ghi đè `.js` được tôn trọng — addon tích hợp giữ manifest riêng của nó.
