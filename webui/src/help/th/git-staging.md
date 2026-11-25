พื้นที่ `CHANGES` ของแผง Git ครอบคลุมวงจรคอมมิตประจำวัน: ทบทวนการเปลี่ยนแปลง stage ทิ้ง stash และ commit พร้อมข้อความ

## Stage และ unstage

- การเปลี่ยนแปลงที่ไม่ได้ stage อยู่ใต้ `CHANGES` แต่ละไฟล์มีป้ายสถานะ — `M` แก้ไข, `A` เพิ่ม, `D` ลบ, `?` ไม่ถูกติดตาม, `!` ความขัดแย้ง (แถว submodule มีแท็ก `SUB` เพิ่ม);
- ชี้ที่แถวจะเผยปุ่มการกระทำในแถว: `+` stage ไฟล์ที่ยังไม่ stage, `−` unstage ที่ stage แล้ว และ `×` บนแถวไม่ stage ทิ้งมัน; ปุ่ม `Stage All Changes` / `Unstage All Changes` ในหัวข้อจัดการทั้งหมดครั้งเดียว;
- `Ctrl`-คลิกสลับไฟล์เข้าการเลือกหลายไฟล์ และ `Shift`-คลิกเลือกช่วง — ปุ่มการกระทำจึงใช้กับทุกไฟล์ที่เลือก;
- คลิกชื่อไฟล์เปิด diff ของมันเพื่อทบทวนก่อนตัดสินใจ;
- คลิกขวาไฟล์มี `View File History`, `Stash This File`, `Copy Full Path` / `Copy Relative Path`, `Reveal in File Explorer` และทางลัด `cd` ที่พาเทอร์มินัลไปโฟลเดอร์ของไฟล์

![ส่วน STAGED CHANGES — ไฟล์ที่ stage พร้อมปุ่ม Unstage รายแถว](images/git-staging-staged.jpg)

## การ commit

1. Stage สิ่งที่ควรเข้า;
2. พิมพ์ข้อความในกล่องคอมมิต (`Ctrl+Enter` คอมมิต) หรือกดปุ่ม AI สร้าง (ดู ผู้ช่วย LLM);
3. คลิก `Commit`

![กล่องคอมมิต — ช่องข้อความพร้อมปุ่มสร้างด้วย AI](images/git-staging-commit.jpg)

หลัง commit ชี้ที่หัวประวัติ — แถบชื่อ branch เหนือประวัติคอมมิต — และปุ่ม `Fetch`, `Pull` และ `Push` จางเข้ามาขวา ๆ `Push` ส่ง branch ปัจจุบันไป upstream ทันที; สำหรับตัวเลือกอย่าง `Force with lease` ใช้กล่อง push ของรายการ branch (ดู Push, Checkout และอื่น ๆ)

![ชี้ที่หัวประวัติเผยปุ่ม Fetch, Pull และ Push](images/git-staging-push.jpg)

## Stash

- ปุ่มหีบห่อในหัว `CHANGES` เปิดกล่อง `Stash All Changes` โดยข้อความไม่บังคับอธิบาย stash;
- คลิกขวาไฟล์ที่ไม่ stage ที่เลือกไว้มี `Stash This File` (หรือ `Stash <n> Selected Files` สำหรับเลือกหลายไฟล์) เพื่อ stash เฉพาะพาธเหล่านั้น — แถว submodule ถูกยกเว้น;
- stash รวมใต้ `STASHES` เป็นรายการ `WIP on <branch>`; ขยายหนึ่งรายการแสดงไฟล์ที่เก็บ และคลิกไฟล์แสดง diff;
- ชี้ที่แถว stash เห็น `Apply` (คืนและเก็บไว้), `Pop` (ใช้และลบ) และ `Drop` (ลบโดยไม่ใช้) — `Pop` และ `Drop` ขอยืนยันก่อน และเมนูคลิกขวาเพิ่ม `View Diff`;
- ส่วน `STASHES` ซ่อนตราบที่รายการ stash ว่าง

![กล่อง Stash All Changes พร้อมข้อความไม่บังคับ](images/git-staging-stash-all.jpg)

![ส่วน STASHES — stash ขยายเป็นไฟล์ พร้อมปุ่ม Apply, Pop และ Drop](images/git-staging-stashes.jpg)

![Pop ใช้ stash และลบออกจากรายการหลังยืนยัน](images/git-staging-stash-pop.jpg)

## การทิ้ง

`Discard Changes` บนไฟล์ที่ไม่ stage ทิ้งการเปลี่ยนแปลงที่ยังไม่คอมมิตทั้งหมดในมัน — มีกล่องยืนยันและการกระทำนี้ **ย้อนกลับไม่ได้**

> คอมมิตรวมเฉพาะเนื้อหาที่ stage; การเปลี่ยนแปลงที่ไม่ stage คงอยู่ในต้นไม้ทำงาน
