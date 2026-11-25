Terminal là cốt lõi của Vibe Terminal: mỗi tab lưu trữ một hoặc nhiều phiên terminal, có thể chia thành các ngăn hoặc kéo thành cửa sổ riêng. Terminal truy cập máy chủ cục bộ cũng như các máy chủ từ xa và chia sẻ qua relay.

## Tạo terminal

- Nhấn nút `+` trên thanh tiêu đề để mở tab với hồ sơ mặc định;
- nhấn hình chevron cạnh `+` cho menu hồ sơ: hồ sơ terminal nhóm theo máy chủ (cục bộ, đã kết nối, chia sẻ) — mở rộng nhóm và chọn hồ sơ để mở terminal trên máy chủ đó;
- kéo một thư mục cục bộ lên thanh tiêu đề và thả để mở tab mới với thư mục làm việc đó (chỉ máy chủ cục bộ; mục thả phải là thư mục).

![Thanh tiêu đề — nút tab mới và menu hồ sơ](images/terminal-usage-creation.jpg)

## Chiếm chuột và menu chuột phải

- Nhấp chuột phải trong terminal mở menu chuột phải của nó: dán, `Split` ngăn, lệnh nhanh, bộ sưu tập biến môi trường, sao chép toàn bộ văn bản, các dòng thiết lập MCP và hơn nữa, tùy phiên;
- Các ứng dụng chiếm chuột (opencode, pi, vim, tmux, htop, …) sẽ nhận các sự kiện chuột thay vì terminal — giữ `Shift` để vẫn thực hiện các thao tác thông thường khi ứng dụng như vậy đang chạy:
  - **Sao chép văn bản**: giữ `Shift` và chọn văn bản, rồi nhấp vào chỗ khác (vẫn giữ `Shift`) để bỏ chọn; vùng chọn được sao chép vào clipboard;
  - **Menu chuột phải**: `Shift` + nhấp chuột phải hoặc `Ctrl` + nhấp chuột phải mở menu chuột phải; nhấp chuột phải trong khi đang giữ nút trái cũng mở được (vùng chọn sẽ bị xóa trước);
  - **Liên kết**: `Shift` + nhấp mở liên kết như bình thường.

## Chia ngăn và tổ chức bằng kéo

- **Chia**: nhấp chuột phải trong terminal → `Split` → `Left` / `Right` / `Top` / `Bottom` chia tab thành các ngăn, mỗi ngăn là một terminal độc lập;

  ![Chia tab — menu Split của chuột phải và các ngăn kết quả](images/terminal-usage-split.gif)

- **Gộp bằng kéo**: kéo một tab lên terminal khác và các vùng neo hiện ra (trái/phải/trên/dưới); thả để gộp tab vào tab đích thành ngăn mới;

  ![Gộp bằng kéo — tab thả lên terminal khác trở thành ngăn mới](images/terminal-usage-merge.mp4)

- **Tách thành cửa sổ**: kéo tab ra khỏi dải tab (hoặc lên cửa sổ khác) và cả tab rời khỏi cửa sổ của nó thành một cái mới. Phiên luôn chạy trên máy chủ — chia, gộp và kéo không bao giờ làm gián đoạn lệnh đang chạy.

## Tab và nhóm tab

- **Tab** — một tab terminal, có thể có vài ngăn chia;
- **Nhóm tab** — một thùng chứa tổ chức tab. Mặc định nhóm theo máy chủ: tab của máy chủ cục bộ nằm trong `Ungrouped`, và mỗi máy chủ chia sẻ nhận một nhóm `Name (Server)`; hỗ trợ nhóm tùy chỉnh;
- **Nhóm và cửa sổ** — mỗi cửa sổ hiển thị một nhóm đang hoạt động qua bộ chọn nhóm trên thanh tiêu đề (hoặc `All`); `Move Group to New Window` xé cả nhóm thành cửa sổ riêng, dành trọn cửa sổ hay màn hình cho một máy;
- **Nhóm và máy chủ** — nhóm nhớ máy chủ của nó, nên chuyển nhóm tái tập trung không gian làm việc của máy chủ; menu hồ sơ theo nhóm đang hoạt động và chỉ hiện hồ sơ của các máy chủ liên quan.

![Bộ chọn nhóm — danh sách nhóm và chuyển sang cửa sổ mới](images/terminal-usage-groups.jpg)
