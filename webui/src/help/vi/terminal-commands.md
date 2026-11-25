Lệnh nhanh ghim các lệnh build/test/triển khai của dự án vào menu chuột phải của terminal — một cú nhấp để chạy, sẵn dùng mọi nơi. Lệnh được duy trì theo kho trong `.vterm/commands.json` tại thư mục gốc kho.

## Tạo commands.json

Nhấp chuột phải trong terminal → `Commands`:

- khi kho của phiên chưa có commands.json, menu đề nghị một mục duy nhất, `Generate commands.json` — nhấp vào khiến máy chủ dò loại dự án (npm / Cargo / .NET / Xcode / Gradle-Android / chung) và ghi một bản khởi động bảo thủ. Tệp không trống sẵn có **không bao giờ bị ghi đè**;
- vị trí tệp: `.vterm/commands.json` tại thư mục gốc kho Git (thư mục làm việc hiện tại cho các thư mục không phải Git).

## Tinh chỉnh

Bản mẫu được tạo chỉ chứa các lệnh thông dụng của loại dự án được dò ra. Hãy để tác tử mã của bạn (vd. một tác tử trình soạn thảo nối với MCP của máy chủ này) đọc dự án và tinh chỉnh tệp:

```json
{
  "commands": [
    { "name": "Build", "command": "cargo build --release" },
    { "name": "Test verbose", "command": "cargo test -- --nocapture",
      "env": { "RUST_LOG": "debug" } },
    { "name": "Package (Windows)", "command": "./packaging/windows/package.bat",
      "os": "win" }
  ]
}
```

Một mảng cấp cao nhất cũng được chấp nhận; `env` tùy chọn đặt biến trước khi lệnh chạy; khóa cũ `value` vẫn được đọc. Trường tùy chọn `os` giới hạn mục theo hệ điều hành đang chạy `vterm-server`: dùng `"win"` cho Windows, `"linux"` cho Linux hoặc `"osx"` cho macOS. Nếu `os` bị bỏ qua, đặt thành `"all"`, hoặc mang giá trị không được nhận dạng, mục có sẵn trên mọi nền tảng. Máy chủ theo dõi CWD và các thay đổi commands.json và phát đi — mở lại menu và lệnh mới đã ở đó.

## Chạy

Chuột phải → `Commands` → nhấp một mục để chạy nó trong terminal đó (env tùy chọn được áp trước). Menu không khả dụng khi một ứng dụng tác tử đang chạy trong phiên, để văn bản lệnh không rơi vào UI của ứng dụng.

## Ví dụ: build & cài Android qua Wi-Fi

Một trình bày trọn vẹn trên dự án Gradle-Android: ghim thiết lập ADB không dây và luồng build-cài vào menu chuột phải, rồi build lại ứng dụng bằng một cú nhấp.

**1. Chuyển điện thoại sang ADB không dây.** Hai lệnh này được gõ một lần mỗi thiết bị — `adb tcpip 5555` khi điện thoại đang cắm USB, rồi `adb connect` với địa chỉ của điện thoại:

![Terminal — chuyển điện thoại Android sang ADB không dây bằng adb tcpip 5555 và adb connect](images/terminal-commands-adb-setup.gif)

**2. Để tác tử mã viết lệnh.** Mở tác tử mã của dự án (ở đây Claude Code) trong terminal và nhờ thêm các lệnh build & cài — nó đọc bố cục dự án và ghi các mục vào `.vterm/commands.json`:

![Terminal — Claude Code chỉnh sửa commands.json với lệnh build & cài APK](images/terminal-commands-agent-edit.mp4)

**3. Build và cài từ menu.** Chia terminal để đầu ra build có ngăn riêng, rồi chuột phải → `Commands` → mục — Gradle đóng gói APK và cài nó lên điện thoại qua Wi-Fi:

![Terminal — chia ngăn và chạy lệnh Android build & cài từ menu chuột phải](images/terminal-commands-split-run.mp4)
