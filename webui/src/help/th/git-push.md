เมนูคลิกขวาของรายการ branch รวมการ push, checkout และการกระทำ branch ประจำวัน

## Push

คลิกขวา branch → `Push '{branch}'...` เปิดกล่อง push:

![คลิกขวา branch — รายการ Push](images/git-push-contextmenu.jpg)

- แสดงการจับคู่เป้าหมาย `local → remote`; แก้ชื่อใดก็ได้ (push ไป branch ระยะไกลที่มีอยู่ด้วยชื่ออื่นจะขอยืนยัน);
- ตัวเลือก: `Push all branches`, `Set upstream` (ผูก upstream ตอน push ครั้งแรก), `Push tags`, `Dry run`;
- `Force` ที่เขียนประวัติใหม่ กับ `Force with lease` ที่ปลอดภัยกว่า (ปฏิเสธเมื่อมีคนอื่น push — แนะนำ) การกระทำที่เขียนประวัติใหม่ เช่น push หลัง rebase ต้องใช้หนึ่งในนี้;
- กล่องแสดงคำเตือนความเสี่ยงของ force push

![กล่อง push — เป้าหมาย local → remote และช่องติ๊กตัวเลือก](images/git-push-dialog.jpg)

## Checkout

คลิกขวา branch → `Checkout` เพื่อสลับ branch — branch ท้องถิ่น checkout ตรง ๆ ส่วน branch ระยะไกลหรือแท็กเปิดกล่อง checkout:

![คลิกขวา branch — รายการ Checkout](images/git-checkout-contextmenu.jpg)

- `New branch name` — branch ท้องถิ่นที่จะสร้าง เติมจาก ref;
- `Create new local branch` (`-B`), `Track remote as upstream` (`--track`, เฉพาะ branch ระยะไกล), `Force` (`-f`), `Merge` (`--merge`)

![กล่อง checkout — ชื่อ branch และตัวเลือก](images/git-checkout-dialog.jpg)

ไดเรกทอรีทำงานของเซสชันตามการสลับ; เทอร์มินัลและแผง Git สะท้อน branch ใหม่ทันที

## Detached HEAD

การ checkout คอมมิตเปล่า (เช่น checkout คอมมิตขณะเลื่อดูประวัติ) ทำ HEAD หลุด คอมมิตถัดไปจะเปิดกล่อง `Detached HEAD`: เลือก branch ที่มีอยู่เพื่อคอมมิต `+ Create new branch…` ในที่ หรือ `Skip` ไปก่อน

## การกระทำอื่น

- **Stash** — stash การเปลี่ยนแปลงทั้งหมดลงกอง (ข้อความไม่บังคับ) จากหัวการเปลี่ยนแปลง; รายการ `STASHES` รองรับ `Pop` (ใช้และลบ) และ `Drop` (ทิ้ง ย้อนกลับไม่ได้);
- **ลบ branch/worktree** — ผ่านเมนูคลิกขวาของรายการ ทั้งคู่มีการยืนยันการกระทำที่ย้อนกลับไม่ได้;
- **ประวัติและการคืนค่า** — `HISTORY` รายคอมมิต; คลิกขวาดูประวัติไฟล์ diff กับต้นไม้ทำงาน หรือคืนค่าไปรีวิชัน;
- **คัดลอก** — ชื่อ branch และแฮชคอมมิตคัดลอกคลิกเดียว สะดวกวางให้เอเจนต์หรือเพื่อนร่วมงาน
