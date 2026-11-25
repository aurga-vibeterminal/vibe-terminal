Worktree checkout branch หลายอันของ repo เดียวไปไดเรกทอรีแยก: พัฒนาขนานโดยไม่แตะต้นไม้ทำงานหลัก แล้ว merge branch กลับและเก็บกวาด Vibe Terminal เปลี่ยนวงจรทั้งหมดเป็นคลิก

## การสร้าง worktree

1. ในรายการ branch ของแผง Git คลิกขวา branch เป้าหมาย (ท้องถิ่นหรือระยะไกล) → `New Worktree`;
2. กล่องตั้งชื่อ worktree ใหม่ตาม branch — แก้ได้ตามชอบ; ยืนยัน แล้วเซิร์ฟเวอร์สร้างไดเรกทอรี worktree และ checkout branch ไปที่นั่น;
3. **แท็บเทอร์มินัลใหม่เปิดอัตโนมัติ** โดยไดเรกทอรีทำงานชี้ที่ worktree ใหม่ — เริ่มพัฒนาแบบแยกทันที ขณะที่ต้นไม้หลักคงอยู่บน branch ของมัน

![กล่อง New Worktree — ชื่อค่าเริ่มต้นคือ branch](images/git-worktree-create.jpg)

![แท็บใหม่ที่เปิดสำหรับ worktree ที่เพิ่งสร้าง](images/git-worktree-terminal.jpg)

## การทำงานขนาน

Worktree แต่ละอันเป็นไดเรกทอรีทำงานอิสระสมบูรณ์: build และทดสอบของ branch หลักกับ branch ฟีเจอร์รันคู่กันในคนละแท็บโดยไม่ชนกัน Branch ที่ checkout ใน worktree มีเครื่องหมาย worktree แสดงพาธในรายการ branch และการคอมมิตกับ push ทำจากแท็บของ worktree ได้เหมือน branch ใด ๆ

คลิกขวา branch worktree เพิ่มสองรายการเฉพาะ: `Reveal in File Explorer` (เซสชันท้องถิ่น) เปิดไดเรกทอรีในตัวจัดการไฟล์ของระบบ และ `Open in Terminal` เปิดอีกแท็บที่นั่น

## Merge กลับและเก็บกวาด

1. คอมมิตและ push ในแท็บ worktree ตามปกติ;
2. สลับแท็บหลักไป branch เป้าหมาย (เช่น `main`) คลิกขวา branch ฟีเจอร์ → `Merge into Current Branch` เพื่อดึงงานมา;

   ![การ merge branch worktree กลับเข้า branch หลัก](images/git-worktree-merge.jpg)

3. เมื่อ merge แล้ว คลิกขวา branch → `Delete Worktree`: ลบไดเรกทอรี worktree และ branch ของมันพร้อมกัน ตรวจก่อนว่าทุกอย่าง merge หรือ push แล้ว — กล่องเตือนซ้ำและการกระทำย้อนกลับไม่ได้

   ![การยืนยัน Delete Worktree — ไดเรกทอรีและ branch ถูกลบพร้อมกัน](images/git-worktree-delete.jpg)

