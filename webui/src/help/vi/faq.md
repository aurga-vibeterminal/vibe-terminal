## Trình duyệt cảnh báo “Kết nối của bạn không phải là kết nối riêng tư” — có bình thường không?

Bình thường. Các cổng được chuyển tiếp, Web UI và các kết nối máy khách dùng chung một chứng chỉ tự ký của máy chủ; bản thân việc mã hóa không bị xâm phạm. Nhập chứng chỉ của máy chủ vào kho chứng chỉ tin cậy của thiết bị để tắt hẳn cảnh báo này.

## Vibe Terminal có miễn phí không?

Có — ứng dụng máy chủ/máy khách desktop và máy chủ headless đều miễn phí. Tải về và cài đặt trên bao nhiêu máy tùy bạn.

## Dữ liệu của tôi đi đâu?

Không đi đâu ngoài nơi bạn tự đặt ra. Phiên làm việc, thông tin đăng nhập (enc-v2:) và nhận dạng giọng nói đều nằm trên máy của bạn; khóa ghép đôi không bao giờ truyền dạng văn bản thuần.

## Có thể công khai an toàn một dịch vụ được chuyển tiếp không?

Có. Các chia sẻ tích hợp yêu cầu token: liên kết do máy khách tạo cho chia sẻ `Host with HTTP server` hay trang Settings đều kèm token truy cập, và kết nối không có token hợp lệ và không có phiên hợp lệ sẽ bị cắt trước khi trả về bất kỳ nội dung nào. Ở lần truy cập đầu tiên, token được đổi lấy cookie phiên bảo mật chỉ áp dụng cho dịch vụ đó và biến mất khỏi URL — ticket của Settings chỉ có tác dụng đúng một lần và hết hạn sau một phút, còn dừng hoặc khởi động lại bản chia sẻ sẽ vô hiệu hóa các liên kết và phiên trước đó. Hãy coi liên kết đầy đủ như một chứng chỉ và chỉ trao cho người bạn tin tưởng. Chuyển tiếp tới dịch vụ riêng của bạn chỉ thêm TLS với chứng chỉ máy chủ chứ không thêm xác thực — hãy giữ lớp xác thực trước các dịch vụ nhạy cảm, và xóa chuyển tiếp ngay lập tức từ bảng điều khiển chuyển tiếp cổng khi dùng xong.
