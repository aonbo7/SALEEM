# SALEEM — نظام الاستخراج العميق للملفات الضخمة والحذف الآمن بتحكم المستخدم
## (`DEEP_LARGE_FILES_SYSTEM.md`)

**الحالة:** النواة الأساسية مفعّلة في `SALEEM.DiskEngine.dll` و`SALEEM.CleanupEngine.dll` — والمرحلة المتقدمة مجدولة للتوسع بعد إتمام مراحل المشروع الأساسية (Post-Phase 7 Deep Expansion).

---

## 1. الهدف الفلسفي والهندسي للنظام
يهدف نظام **الاستخراج العميق للملفات الضخمة (Deep Large File Intelligence & User-Controlled Safe Removal)** في **SALEEM** إلى حل مشكلة امتلاء الأقراص الخفي في أنظمة Windows 10/11 بسبب ملفات ضخمة منسية أو مدفونة في مسارات عميقة داخل مجلدات المستخدم أو الألعاب أو حاويات التطوير، مع الالتزام الصارم بقاعدة **تحكم المستخدم الكامل وعدم الحذف التلقائي**.

---

## 2. ما تم بناؤه وتفعيله حالياً (الجيل الأول — Core Deep Extractor)

### 2.1 الخوارزمية الحالية في `SALEEM.DiskEngine.dll` (`crates/saleem_disk_engine/src/scanner.rs`)
1. **مسح شجري عميق غير عودي (Iterative Stack Traversal):**
   - يغوص المحرك في جميع المستويات الفرعية للمسار المحدد مهما بلغ عمق المجلدات (`depth = ∞`) دون خطر امتلاء مكدس الذاكرة (`Stack Overflow`).
   - يفحص سمات نظام الملفات عبر `symlink_metadata` ويتخطى نقاط إعادة التحليل (`Reparse Points` / `NTFS Junctions` / `Symlinks`) لمنع الحلقات اللانهائية.
2. **كومة الذاكرة المحدودة $O(K)$ (`BinaryHeap<MinHeapFileItem>`):**
   - يحتفظ المحرك بأكبر $K$ ملف (تصل إلى `500` ملف ضخم في المسح الواحد) باستخدام هيكل بيانات `Min-Heap` عكسي، مما يسمح بفحص ملايين الملفات بسرعة عالية وبصمة ذاكرة ثابتة لا تتجاوز بضعة ميجابايتات.
3. **حساب الحجم المنطقي مقابل الحجم الفعلي المحجوز على القرص (`Logical` vs. `Allocated Size`):**
   - يستدعي المحرك دالة نظام ويندوز الرسمية `GetCompressedFileSizeW` لمعرفة عدد الكتل الفيزيائية (`NTFS Clusters`) التي يستهلكها الملف فعلياً على القرص الصلب، وليس مجرد الحجم الاسمي.
4. **التصنيف العميق للفئات (`classify_extension_category`):**
   - يصنف المحرك الملفات الضخمة إلى فئات دقيقة تشمل:
     - **الأقراص الافتراضية وصور الأنظمة (`Virtual Disks & OS Images`):** `.vhdx`, `.vhd`, `.vmdk`, `.vdi`, `.qcow2`, `.iso`, `.wim`, `.esd`.
     - **أصول وحزم الألعاب (`Games & Engine Assets`):** `.pak`, `.ucas`, `.utoc`, `.vpk`, `.rpf`, `.bundle`, `.ba2`, `.pck`.
     - **نماذج الذكاء الاصطناعي والأوزان (`AI Models & Weights`):** `.gguf`, `.safetensors`, `.onnx`, `.pt`, `.pth`, `.ckpt`.
     - **ملفات تفريغ الذاكرة والأعطال (`Logs, Temp & Crash Dumps`):** `.dmp`, `.hdmp`, `.mdmp`, `.etl`, `.log`.
     - **قواعد البيانات والنسخ الاحتياطية (`Databases & Backups`):** `.sqlite`, `.sqlite3`, `.mdf`, `.ldf`, `.bak`, `.parquet`.
     - **الوسائط الخام والفيديو (`Video & Audio Media`):** `.mp4`, `.mkv`, `.mov`, `.avi`, `.raw`.
     - **الأرشيفات المضغوطة (`Archives & Compressed`):** `.zip`, `.rar`, `.7z`, `.tar`, `.gz`, `.zst`.
5. **تحديد المستخدم والحذف الآمن المحقق (`User-Controlled Verified Removal`):**
   - يستطيع المستخدم من تبويب **«أكبر الملفات» (Large Files Explorer)** فلترة الملفات حسب الحد الأدنى للحجم (`10 MB`, `50 MB`, `100 MB`, `500 MB`, `1 GB`) أو الفئة، وتحديد الملفات التي يرغب بحذفها يدوياً.
   - يظهر **شريط الإجراءات الزجاجي العائم (`Floating Glossy Action Dock`)** في أسفل الشاشة ليعرض إجمالي عدد الملفات المحددة والمساحة التي سيتم تحريرها.
   - عند ضغط المستخدم على تأكيد الحذف، يمر كل ملف عبر بوابات الحماية في `validate_and_execute_cleanup`:
     1. **حاجز مسارات النظام المحمية (`assert_safe_for_modification`):** منع قاطع لأي ملف يقع داخل `WinSxS` أو `System32` أو `DriverStore` أو `System Volume Information`.
     2. **فحص القفل النشط (`is_file_currently_locked`):** محاولة فتح الملف بوضع عدم المشاركة (`CreateFileW` مع `dwShareMode = 0`)؛ إذا كان الملف مفتوحاً من أي برنامج آخر، يتم تخطيه بأمان دون إجبار أو إسقاط للنظام.
     3. **التحقق من عدم التغير (`TOCTOU Size Check`):** التأكد من أن حجم الملف لم يتغير منذ لحظة الفحص وحتى لحظة الحذف.
     4. **التحقق البعدي والتوثيق (`Post-Delete Verification & SQLite Audit`):** التأكد من اختفاء الملف فعلياً من القرص وتسجيل العملية في جداول `cleanup_jobs` و`cleanup_items` و`audit_events`.

---

## 3. خارطة طريق التوسع العميق بعد إتمام الفيزات الأساسية (Post-Phase 7 Deep Expansion)

بعد الانتهاء من الفيزات الأساسية للمشروع (من المرحلة 4 إلى المرحلة 7) والاطمئنان على استقرار كافة المحركات، سنعود لتوسيع هذا النظام بالمزايا المتقدمة التالية:

### الميزة المستقبلية 1: مؤشر الخمول الزمني للملفات الضخمة (`Staleness & Cold-File Score`)
- تحليل تاريخ آخر تعديل (`LastWriteTime`) وتاريخ آخر وصول (`LastAccessTime` إن كان مفعلاً في NTFS) لحساب **درجة خمول الملف (Cold Score)**.
- إبراز الملفات الضخمة (أكبر من `1 GB`) التي لم يفتحها المستخدم منذ أكثر من `90` أو `180` يوماً بشارة خاصة: `ملف ضخم خامل منذ +6 أشهر`.

### الميزة المستقبلية 2: الفحص العميق لحاويات التطوير والأقراص الافتراضية (`Deep Container & VHDX Inspector`)
- اكتشاف ملفات الأقراص الافتراضية المتضخمة مثل:
  - `ext4.vhdx` الخاص بـ **WSL2** و**Docker Desktop** (الذي يتمدد تلقائياً ولا يتقلص بعد حذف الملفات داخله).
  - مجلدات بناء المطورين العميقة والضخمة (`target/` في Rust، `node_modules/`، `.gradle/`، `DerivedData/`، كاش `pip` و`huggingface`).
- تقديم توصية ذكية للمستخدم توضح الفرق بين **حذف الملف** أو **ضغط/تقليص القرص الافتراضي (Compact VHDX)**.

### الميزة المستقبلية 3: معاينة محتوى الأرشيفات الضخمة قبل الحذف (`Archive Peek without Extraction`)
- عندما يجد المحرك ملف `.zip` أو `.7z` أو `.tar` بحجم عدة جيجابايتات، يقرأ جدول الفهرس الداخلي للأرشيف (Central Directory Header) دون فك ضغطه، ويعرض للمستخدم أهم 5 ملفات بداخله حتى يتأكد المستخدم من محتوى الأرشيف قبل أن يقرر حذفه.

### الميزة المستقبلية 4: خيار «النقل إلى الحجر الآمن أولاً» (`Move Giant File to Quarantine Before Delete`)
- إتاحة زرين للمستخدم عند تحديد ملفات كبيرة:
  1. **نقل إلى الحجر المؤقت (Quarantine Vault):** نقل الملف على نفس القرص (`MoveFileExW` فوري بدون نسخ بطيء) إلى خزنة العزل بحيث يمكن استعادته بضغطة زر إذا احتاجه المستخدم لاحقاً.
  2. **حذف نهائي محقق (Verified Permanent Removal):** الحذف المباشر بعد التحقق من السياسات وبصمة الحجم.

### الميزة المستقبلية 5: كاشف الملفات المتناثرة والمضغوطة (`NTFS Sparse & Compression Inspector`)
- مقارنة دقيقة بين `Logical Size` و`Allocated Size`:
  - إذا كان `Allocated Size < Logical Size`، يوضح التطبيق للمستخدم أن الملف مضغوط عبر NTFS أو ملف متناثر (`Sparse File`)، ويعرض **المساحة الفيزيائية الحقيقية** التي ستتحرر فعلاً عند حذفه.

---

## 4. ارتباط النظام بالملفات البرمجية الحالية
- المحرك الخلفي للمسح والاستخراج: [`crates/saleem_disk_engine/src/scanner.rs`](file:///d:/sleem/crates/saleem_disk_engine/src/scanner.rs)
- محرك التحقق والحذف الآمن: [`crates/saleem_cleanup_engine/src/engine.rs`](file:///d:/sleem/crates/saleem_cleanup_engine/src/engine.rs)
- سياسة حماية مسارات النظام: [`crates/saleem_core/src/paths.rs`](file:///d:/sleem/crates/saleem_core/src/paths.rs)
- واجهة المستخدم التفاعلية: [`apps/desktop/src/App.tsx`](file:///d:/sleem/apps/desktop/src/App.tsx)
