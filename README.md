# 🛡️ SALEEM — Windows Disk Intelligence, Kernel Memory Reduction & Static Security Suite

**SALEEM** هو تطبيق سطح مكتب احترافي لنظام **Windows 10 / 11 (x64)** مبني باستخدام **Tauri 2 + React 18 + TypeScript + Rust + SQLite (WAL)** مع **4 محركات أصلية مستقلة (`Native/*.dll`)** تعمل عبر واجهة `C-ABI`.

---

## ✨ المميزات والمحركات المستقلة (Core Features & Native Engines)

| المحرك الأصلي (DLL) | الوظيفة الأساسية |
| :--- | :--- |
| **`Native/SALEEM.DiskEngine.dll`** | فحص شجري متوازٍ للأقراص والمجلدات، حساب الحجم المنطقي والمخصص (`GetCompressedFileSizeW`)، خريطة المساحة التفاعلية (Bubble & Treemap)، واكتشاف الملفات المكررة عبر التجزئة المرحلية (`BLAKE3` + `SHA-256`). |
| **`Native/SALEEM.CleanupEngine.dll`** | محرك التنظيف الآمن ذو الـ 9 مراحل مع حماية صارمة لمسارات النظام الحرجة (`WinSxS`, `System32`, `DriverStore`) ونقل آمن إلى خزنة الحجر (`Quarantine Vault`). |
| **`Native/SALEEM.MemoryEngine.dll`** | محرك تفريغ الرامات الخارق المستقل (`MemReduct Pro` + `NT Kernel`): تفريغ مجموعات العمل (`Working Sets`)، قوائم الانتظار (`Standby List`)، الصفحات المعدلة (`Modified Page List`)، كاش ملفات النظام، ودمج الصفحات الفيزيائية المتطابقة مع مراقب خلفي ذكي وإشعارات Windows فورية. |
| **`Native/SALEEM.Sentinel.dll`** | درع الفحص الأمني الثابت (Static PE Analysis & Digital Signature Verification) دون تشغيل الملفات المشبوهة. |

---

## ⚙️ إعداد المشروع والتعديل عليه (Configuration & Setup Guide)

عند تحميل المشروع من GitHub، لا يحتوي الكود على أي مفاتيح أو روابط قواعد بيانات خاصة. لتشغيل نظام التحديثات السحابي (Supabase Cloud Updater) الخاص بك:

### 1. إعداد ملف البيئة `.env`
انسخ ملف القالب `.env.example` إلى ملف جديد باسم `.env` في جذر المشروع:

```powershell
Copy-Item .env.example .env
```

ثم افتح ملف `.env` وعدّل القيم الخاصة بمشروعك على **Supabase**:

```env
SALEEM_SUPABASE_PROJECT_URL=https://YOUR_PROJECT_REF.supabase.co
SALEEM_SUPABASE_POOLER_URI=host=YOUR_REGION.pooler.supabase.com port=5432 user=saleem_readonly.YOUR_PROJECT_REF password=YOUR_READONLY_PASSWORD dbname=postgres connect_timeout=10
SUPABASE_POOLER_HOST=YOUR_REGION.pooler.supabase.com
SUPABASE_ADMIN_USER=postgres.YOUR_PROJECT_REF
```

> **ملاحظة أمنية:** ملف `.env` مضاف داخل `.gitignore` ولن يتم رفعه إلى GitHub أبداً.

---

### 2. متطلبات التشغيل والبناء (Prerequisites)
- **Windows 10 / 11 (x64)**
- **Node.js 20+** & `npm`
- **Rust Stable Toolchain** (`cargo`, `rustc`)
- **NSIS (Nullsoft Scriptable Install System)** (لبناء المثبت الرسمي `SALEEM-Setup.exe`)

---

### 3. تشغيل المشروع في وضع التطوير (Development)

```powershell
cd apps/desktop
npm install
npm run tauri dev
```

---

### 4. بناء النسخة النهائية والمثبت الرسمي (`release/`)

لبناء الواجهة الأمامية + المحركات الأربعة (`Native/*.dll`) + التطبيق التنفيذي (`SALEEM.exe`) + المثبت الرسمي (`SALEEM-Setup.exe`):

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

ستجد المخرجات المنظمة داخل مجلد `release/`:
- `release/SALEEM-Setup.exe` — المثبت الرسمي للمستخدمين
- `release/SALEEM.exe` — الملف التنفيذي المباشر
- `release/Native/SALEEM.DiskEngine.dll`
- `release/Native/SALEEM.CleanupEngine.dll`
- `release/Native/SALEEM.MemoryEngine.dll`
- `release/Native/SALEEM.Sentinel.dll`

---

### 5. نشر تحديث جديد للمستخدمين عبر Supabase (Optional)

بعد ضبط ملف `.env` وبناء المشروع، يمكنك رفع التحديث لقاعدة بيانات Supabase الخاصة بك عبر:

```powershell
$env:SUPABASE_DB_PASSWORD = "YOUR_SUPABASE_POSTGRES_ADMIN_PASSWORD"
powershell -ExecutionPolicy Bypass -File .\scripts\publish-update.ps1 `
  -Version "0.5.0" `
  -DownloadUrl "supabase://app_release_binaries/0.5.0" `
  -NotesAr "تفاصيل ومميزات التحديث بالعربية" `
  -NotesEn "Release notes and features in English"
```

---

## 📄 الترخيص (License)
مرخص تحت رخصة **MIT License** — راجع ملف [`LICENSE`](LICENSE).
