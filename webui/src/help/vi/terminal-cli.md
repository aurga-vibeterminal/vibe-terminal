`vtermc` là máy khách dòng lệnh: nó kết nối tới máy chủ, tạo hoặc gắn vào phiên terminal và quản lý danh sách máy chủ đã lưu — tất cả trong một terminal thường, qua SSH, hoặc trên máy headless. Nó chia sẻ `~/.vibeterminal/client.json` với máy khách desktop nên cả hai thấy cùng máy chủ.

## Khởi động terminal

Chạy `vtermc` không có cờ chế độ để tạo terminal mới trên máy chủ cục bộ (mặc định `wss://localhost:5153`) với shell được phát hiện, và phát nó vào terminal hiện tại. Gõ `exit` (hoặc `Ctrl+D`) trong shell từ xa để kết thúc phiên.

Phiên sống trên máy chủ: nếu kết nối đứt — mất mạng, đóng cửa sổ, giết `vtermc` — phiên vẫn chạy và có thể gắn lại sau bằng `vtermc -s`.

- `vtermc -p <id>` — khởi động với một hồ sơ terminal cụ thể (danh sách hồ sơ nằm trong mục `Profiles` của Web UI);
- `vtermc -p` — không có id hồ sơ: bộ chọn liệt kê hồ sơ của máy chủ tương tác;
- `vtermc -w <dir>` — đặt thư mục làm việc của phiên mới.

Khi stdin không phải terminal (ống hoặc tệp), `vtermc` chuyển sang chế độ theo lô: stdin được đưa vào phiên và đầu ra chảy cho đến khi phiên đóng — tiện từ script hoặc tác tử:

```sh
echo "make test" | vtermc
```

## Kết nối tới máy chủ

- `--server wss://host:port` — kết nối tới máy chủ cụ thể thay vì cục bộ (địa chỉ IPv6 trong ngoặc: `wss://[fd00::1]:5153`); biến môi trường `VTERM_SERVER` đặt cùng mặc định;
- Bắt tay được trả lời bằng khóa bí mật của máy chủ: máy chủ thêm qua `-a`/`-s` mang khóa trong `client.json`; máy chủ cục bộ dự trữ về biến môi trường `TERMINAL_GUI_SECRET_KEY` (hoặc khóa văn bản thuần trong `server.json`).

## Thêm máy chủ

`-a` lưu máy chủ vào `client.json` và thoát — nó không bao giờ mở terminal. Hai cách:

- `vtermc -a` — nhắc tương tác: `Server name`, `Server address` và `Secret key`. Địa chỉ phải là `wss://host:port` (đầu vào không hợp được hỏi lại; địa chỉ trống hủy). Tên trống dự trữ về địa chỉ;
- `vtermc -a "vtshare1:..."` — dán chuỗi chia sẻ được mã hóa. Nhấp `QR Code` ghép nối trong mục `Server` của Web UI sao chép nó vào clipboard; `vtermc` giải mã và tự điền tên, địa chỉ và khóa bí mật.

Chi tiết đáng biết:

- thêm lại địa chỉ đã có cập nhật mục đó tại chỗ (giữ id) thay vì nhân đôi;
- địa chỉ cục bộ bị từ chối — `Local` luôn ngầm định sẵn và không bao giờ cần mục;
- các mục cũng được Server Manager của máy khách desktop hiển thị.

## Chọn máy chủ và phiên

`vtermc -s` mở hai bộ chọn toàn màn hình (cần terminal tương tác):

1. **Danh sách máy chủ** — trước hết `Local`, rồi mỗi máy chủ được quản lý dạng `Name  (địa chỉ)`. Dẫn hướng bằng phím mũi tên hoặc `j`/`k`, nhảy bằng chữ số, `Enter` chọn, `q` hủy;
2. **Danh sách terminal** — trước hết `New Terminal`, theo sau là các phiên sống của máy chủ đó kèm thư mục làm việc, tiêu đề và máy chủ sở hữu. Gắn vào phiên sẵn có, hoặc tạo mới (khi đó tôn trọng `-p` / `-w` như chạy `vtermc` thường).

Gắn vào không bao giờ làm gián đoạn phiên — nó vẫn chạy trên máy chủ giữa các lần gắn.

### Xóa máy chủ

Trong danh sách máy chủ, di chuyển tới máy chủ được quản lý và nhấn `d`: hàng đỏ lên và gợi ý xin xác nhận. Nhấn `y` (hoặc `Enter`) để xóa mục khỏi `client.json`, phím khác hủy. `Local` không thể xóa.

## Biến môi trường

Phiên mới kế thừa một bộ cố định từ shell của bạn — `PATH`, `HOME`, `USER`, `SHELL`, `TERM`, `WT_PROFILE_ID`, `PSMODULEPATH`:

- `--env-inherit <patterns>` — danh sách phân tách bằng dấu phẩy (hoặc biến môi trường `VTERM_ENV_INHERIT`); hỗ trợ wildcard `*`;
- `-e VAR=VALUE` — tiêm biến bổ sung (lặp lại được);
- `--no-env` — khởi động phiên không kế thừa biến nào.

```sh
vtermc -w ~/project -e RUST_LOG=debug --env-inherit "PATH,HOME,TERM"
```

## Mọi tùy chọn

```text
-p, --profile [<ID>]          Chọn hồ sơ terminal (bỏ trống để liệt kê tương tác)
-w, --working-dir <DIR>       Đặt thư mục làm việc
-s, --select                  Chọn máy chủ và terminal sẵn có tương tác
-a, --add [<SHARE>]           Thêm máy chủ (vtshare1: chuỗi chia sẻ, hoặc tương tác)
    --server <URL>            URL WebSocket của máy chủ [env: VTERM_SERVER]
-e, --env <VAR=VALUE>         Biến môi trường bổ sung
    --no-env                  Không kế thừa biến môi trường
    --env-inherit <PATTERNS>  Mẫu biến môi trường cần kế thừa [env: VTERM_ENV_INHERIT]
```
