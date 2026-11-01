import { LanguageCode, NavSectionId } from "../types/saleem";

export interface Translations {
  appName: string;
  appSubtitle: string;
  sidebarGroupMain: string;
  sidebarGroupFiles: string;
  nav: Record<NavSectionId, string>;
  header: {
    refresh: string;
    languageToggle: string;
    themeDark: string;
    themeLight: string;
    phaseBadge: string;
  };
  overview: {
    readyBadge: string;
    scanningBadge: string;
    heroHeadlineNoScan: string;
    heroHeadlineScanned: string;
    heroDescNoScan: string;
    heroDescScanned: string;
    startScanCta: string;
    exploreMapCta: string;
    spaceMapTitle: string;
    spaceMapOpen: string;
    spaceMapHint: string;
    bentoStorageTitle: string;
    bentoStorageSub: string;
    bentoStorageBtn: string;
    bentoLargeTitle: string;
    bentoLargeSub: string;
    bentoLargeBtn: string;
    bentoSentinelTitle: string;
    bentoSentinelSub: string;
    bentoSentinelBtn: string;
    bentoCleanupTitle: string;
    bentoCleanupSub: string;
    bentoCleanupBtn: string;
    enginesTitle: string;
    abiVersion: string;
    loadMode: string;
    databaseTitle: string;
    schemaVersion: string;
    tablesCount: string;
    auditEventsCount: string;
    activeQuarantineCount: string;
    usedSpace: string;
    freeSpace: string;
    totalSpace: string;
  };
  analyzer: {
    title: string;
    subtitle: string;
    customPathPlaceholder: string;
    scanBtn: string;
    cancelBtn: string;
    scanningTitle: string;
    filesSeen: string;
    foldersSeen: string;
    logicalTotal: string;
    allocatedTotal: string;
    partialScanTitle: string;
    partialScanDesc: string;
    reparseSkipped: string;
    viewBubbles: string;
    viewTreemap: string;
    spaceMapSection: string;
    categoriesSection: string;
    extensionsSection: string;
    foldersSection: string;
    searchPlaceholder: string;
    sortByLogical: string;
    sortByAllocated: string;
    sortByFiles: string;
    sortByName: string;
    openExplorer: string;
    copyPath: string;
    copied: string;
    drillDown: string;
    upOneLevel: string;
    emptyStateTitle: string;
    emptyStateDesc: string;
  };
  largeFiles: {
    title: string;
    subtitle: string;
    allCategories: string;
    allSizes: string;
    searchPlaceholder: string;
    colSelect: string;
    colName: string;
    colCategory: string;
    colLogicalSize: string;
    colAllocatedSize: string;
    colModified: string;
    colActions: string;
    noScanYet: string;
    selectedCount: string;
    reviewDeleteBtn: string;
    confirmDeleteBtn: string;
    confirmWarning: string;
    deleteSuccessBanner: string;
  };
  cleanup: {
    title: string;
    subtitle: string;
    scanBtn: string;
    cancelBtn: string;
    scanningTitle: string;
    pipelineBadge: string;
    rulesSectionTitle: string;
    candidatesSectionTitle: string;
    selectAllSafe: string;
    clearSelection: string;
    totalCandidates: string;
    lockedSkipped: string;
    reclaimableBytes: string;
    riskLow: string;
    riskMedium: string;
    floatingSelectedCount: string;
    floatingReclaimable: string;
    reviewCleanBtn: string;
    confirmCleanBtn: string;
    cancelConfirmBtn: string;
    confirmWarning: string;
    cleanSuccessBanner: string;
    emptyStateTitle: string;
    emptyStateDesc: string;
    zeroCandidatesTitle: string;
    zeroCandidatesDesc: string;
  };
  duplicates: {
    title: string;
    subtitle: string;
    scanBtn: string;
    cancelBtn: string;
    scanningTitle: string;
    minSizeLabel: string;
    smartSelectOldest: string;
    smartSelectNewest: string;
    clearSelection: string;
    groupsCount: string;
    reclaimableSpace: string;
    fullHashesVerified: string;
    eachFileSize: string;
    copiesBadge: string;
    retainedBadge: string;
    markedForDeleteBadge: string;
    hardlinkBadge: string;
    hardlinkWarning: string;
    keepThisCopy: string;
    selectForRemoval: string;
    lockedRetainedHint: string;
    floatingSelectedCount: string;
    floatingReclaimable: string;
    reviewAndRemoveBtn: string;
    confirmRemoveBtn: string;
    cancelConfirmBtn: string;
    confirmModalWarning: string;
    removalSuccessBanner: string;
    emptyStateTitle: string;
    emptyStateDesc: string;
    noDuplicatesFoundTitle: string;
    noDuplicatesFoundDesc: string;
  };
  applications: {
    title: string;
    subtitle: string;
    scanBtn: string;
    cancelBtn: string;
    scanningTitle: string;
    tabLeftovers: string;
    tabInstalled: string;
    totalApps: string;
    totalGames: string;
    totalLeftovers: string;
    highConfidenceBytes: string;
    filterAll: string;
    filterHigh: string;
    filterMedium: string;
    filterLow: string;
    filterSavedProtected: string;
    selectHighConfidence: string;
    clearSelection: string;
    confidenceHigh: string;
    confidenceMedium: string;
    confidenceLow: string;
    savedDataProtectedBadge: string;
    lockedFilesBadge: string;
    estimatedSizeBadge: string;
    exactSizeBadge: string;
    searchAppsPlaceholder: string;
    searchLeftoversPlaceholder: string;
    sourceAll: string;
    sourceRegistry: string;
    sourceSteam: string;
    sourceEpic: string;
    colAppName: string;
    colPublisher: string;
    colVersion: string;
    colSource: string;
    colSize: string;
    colActions: string;
    floatingSelectedCount: string;
    floatingReclaimable: string;
    reviewRemoveBtn: string;
    confirmRemoveBtn: string;
    cancelConfirmBtn: string;
    confirmWarning: string;
    removeSuccessBanner: string;
    emptyStateTitle: string;
    emptyStateDesc: string;
    zeroLeftoversTitle: string;
    zeroLeftoversDesc: string;
  };
  history: {
    title: string;
    subtitle: string;
    empty: string;
    colTime: string;
    colType: string;
    colSubject: string;
    colResult: string;
    colDetails: string;
  };
  settings: {
    title: string;
    subtitle: string;
    languageLabel: string;
    themeLabel: string;
    safetyPolicyTitle: string;
    safetyPolicyDesc: string;
  };
  phasePlaceholder: {
    scheduledTitle: string;
    scheduledDesc: string;
  };
}

export const TRANSLATIONS: Record<LanguageCode, Translations> = {
  ar: {
    appName: "سَليم — SALEEM",
    appSubtitle: "ذكاء التخزين والأمان الرقمي لنظام Windows",
    sidebarGroupMain: "الذكاء والتحكم",
    sidebarGroupFiles: "الملفات والحماية",
    nav: {
      overview: "المسح الذكي",
      storage: "خريطة المساحة",
      large_files: "أكبر الملفات",
      duplicates: "الملفات المكررة",
      cleanup: "التنظيف الآمن",
      applications: "البرامج والألعاب",
      startup: "مسرّع النظام",
      memory: "مُفرّغ الرامات الخارق",
      sentinel: "درع Sentinel",
      quarantine: "الحجر والاستعادة",
      history: "سجل التدقيق",
      settings: "الإعدادات",
    },
    header: {
      refresh: "تحديث",
      languageToggle: "English (LTR)",
      themeDark: "الوضع الليلي",
      themeLight: "الوضع المضيء",
      phaseBadge: "المرحلة 4: جرد البرامج والألعاب وبقايا الإزالة + واجهة Crystal Glass",
    },
    overview: {
      readyBadge: "المحركات جاهزة • قراءة حية من النظام",
      scanningBadge: "جاري تحليل شجرة الملفات...",
      heroHeadlineNoScan: "إجمالي المساحة المستخدمة على أقراصك",
      heroHeadlineScanned: "تم تحليل المسار واكتشاف توزيع المساحة",
      heroDescNoScan:
        "محرك SALEEM Disk Engine جاهز لتحليل المجلدات وحساب الحجم المنطقي والفعلي (Allocated) واكتشاف أكبر الملفات دون تجميد الواجهة.",
      heroDescScanned:
        "تستطيع استكشاف خريطة الفقاعات أو خريطة Treemap التفاعلية، والغوص داخل أي مجلد، أو مراجعة أكبر الملفات مباشرة.",
      startScanCta: "بدء فحص سريع",
      exploreMapCta: "فتح خريطة المساحة التفاعلية ←",
      spaceMapTitle: "خريطة المساحة (Space Map)",
      spaceMapOpen: "توسيع الخريطة ‹",
      spaceMapHint: "انقر على أي فقاعة للغوص داخل المجلد أو القرص",
      bentoStorageTitle: "خريطة الأقراص",
      bentoStorageSub: "تحليل شجري متوازٍ مع كشف الروابط الرمزية والنتائج الجزئية",
      bentoStorageBtn: "استكشاف ←",
      bentoLargeTitle: "أكبر الملفات",
      bentoLargeSub: "أضخم الملفات المكتشفة مرتبة حسب الحجم المنطقي والمخصص",
      bentoLargeBtn: "مراجعة الملفات ←",
      bentoSentinelTitle: "حماية Sentinel",
      bentoSentinelSub: "فحص ثابت للبنية والتوقيعات الرقمية دون تشغيل الملفات",
      bentoSentinelBtn: "حالة المحرك ←",
      bentoCleanupTitle: "التنظيف الآمن",
      bentoCleanupSub: "حماية صارمة لمجلدات WinSxS وSystem32 وDriverStore",
      bentoCleanupBtn: "السياسات ←",
      enginesTitle: "حالة المحركات الأصلية (Native DLLs & C-ABI)",
      abiVersion: "إصدار ABI",
      loadMode: "وضع الربط",
      databaseTitle: "قاعدة بيانات التدقيق (SQLite WAL)",
      schemaVersion: "إصدار المخطط",
      tablesCount: "الجداول المفعلة",
      auditEventsCount: "سجلات التدقيق",
      activeQuarantineCount: "عناصر معزولة",
      usedSpace: "المستخدم",
      freeSpace: "الحر",
      totalSpace: "السعة",
    },
    analyzer: {
      title: "محلل المساحة وخريطة الأقراص التفاعلية",
      subtitle:
        "اختر قرصاً أو مجلداً لتحليله بدقة مع التفريق بين الحجم المنطقي (Logical) والمساحة المحجوزة فعلياً على القرص (Allocated)",
      customPathPlaceholder: "أدخل مسار مجلد أو قرص للفحص (مثال: D:\\ أو C:\\Users)...",
      scanBtn: "فحص المسار",
      cancelBtn: "إيقاف الفحص فوراً",
      scanningTitle: "يقوم محرك SALEEM Disk Engine بمسح المجلدات الآن...",
      filesSeen: "ملف تم فحصه",
      foldersSeen: "مجلد",
      logicalTotal: "الحجم المنطقي (Logical)",
      allocatedTotal: "المساحة الفعلية (Allocated)",
      partialScanTitle: "نتيجة جزئية (Partial Scan)",
      partialScanDesc:
        "تعذر قراءة بعض المجلدات الفرعية بسبب صلاحيات Windows (Access Denied). الأرقام المعروضة تمثل الملفات المتاحة فقط.",
      reparseSkipped: "روابط رمزية/Junctions تم تخطيها لمنع الدوران",
      viewBubbles: "خريطة الفقاعات (Bubbles)",
      viewTreemap: "خريطة المساحة (Treemap)",
      spaceMapSection: "التوزيع البصري للمجلدات (انقر على أي عنصر للدخول إليه)",
      categoriesSection: "التوزيع حسب فئة الملفات",
      extensionsSection: "أكثر الامتدادات استهلاكاً للمساحة",
      foldersSection: "المجلدات الفرعية المكتشفة",
      searchPlaceholder: "ابحث باسم المجلد أو المسار...",
      sortByLogical: "ترتيب: الحجم المنطقي",
      sortByAllocated: "ترتيب: المساحة الفعلية",
      sortByFiles: "ترتيب: عدد الملفات",
      sortByName: "ترتيب: الاسم",
      openExplorer: "فتح في Explorer",
      copyPath: "نسخ المسار",
      copied: "تم النسخ!",
      drillDown: "فحص هذا المجلد",
      upOneLevel: "المجلد الأب ↑",
      emptyStateTitle: "اختر مساراً أو قرصاً لبدء التحليل",
      emptyStateDesc:
        "اضغط على أحد الاختصارات السريعة بالأعلى أو أدخل أي مسار لبناء خريطة المساحة واكتشاف أكبر الملفات والمجلدات.",
    },
    largeFiles: {
      title: "مستكشف أكبر الملفات العميق (Deep Large Files Intelligence)",
      subtitle:
        "استخراج عميق لأكبر الملفات على القرص مع تصنيف متقدم وإمكانية تحديد المستخدم للملفات المراد حذفها بأمان",
      allCategories: "جميع الفئات",
      allSizes: "كل الأحجام",
      searchPlaceholder: "ابحث باسم الملف أو الامتداد أو المسار...",
      colSelect: "تحديد",
      colName: "اسم الملف والمسار",
      colCategory: "الفئة والامتداد",
      colLogicalSize: "الحجم المنطقي",
      colAllocatedSize: "الحجم المخصص (Allocated)",
      colModified: "تاريخ التعديل",
      colActions: "إجراءات",
      noScanYet:
        "يرجى تشغيل فحص من «المسح الذكي» أو «خريطة المساحة» أولاً لاستخراج أكبر الملفات بعمق.",
      selectedCount: "ملفات ضخمة محددة للحذف",
      reviewDeleteBtn: "مراجعة وحذف الملفات المحددة",
      confirmDeleteBtn: "تأكيد الحذف النهائي المحقق",
      confirmWarning:
        "سيقوم SALEEM بالتحقق من عدم قفل الملف ومن سياسة مسارات النظام المحمية ومطابقة الحجم قبل حذفه وتوثيق العملية في SQLite.",
      deleteSuccessBanner: "تم تنفيذ حذف الملفات الضخمة المحددة وتوثيقها في سجل التدقيق:",
    },
    cleanup: {
      title: "محرك التنظيف الآمن المبني على القواعد (SALEEM Cleanup Engine)",
      subtitle:
        "خط أنابيب من 9 مراحل (فحص ← تحليل ← تخطيط ← معاينة ← تأكيد ← تحقق ← تنفيذ ← إثبات ← تقرير) مع حماية الملفات قيد الاستخدام",
      scanBtn: "فحص عناصر التنظيف الآمن",
      cancelBtn: "إيقاف الفحص",
      scanningTitle: "يقوم محرك SALEEM Cleanup Engine بفحص مسارات الكاش والملفات المؤقتة...",
      pipelineBadge: "خط أنابيب 9 مراحل • بدون حذف تلقائي",
      rulesSectionTitle: "قواعد التنظيف النشطة (اختر الفئات المراد فحصها)",
      candidatesSectionTitle: "معاينة الملفات المرشحة للتنظيف (قابلة للتحديد اليدوي)",
      selectAllSafe: "تحديد جميع العناصر الآمنة (Low Risk)",
      clearSelection: "إلغاء التحديد",
      totalCandidates: "عناصر قابلة للتنظيف",
      lockedSkipped: "ملفات مفتوحة تم تخطيها بأمان",
      reclaimableBytes: "مساحة قابلة للتحرير",
      riskLow: "آمن (Low Risk)",
      riskMedium: "يُنصح بالمراجعة (Medium)",
      floatingSelectedCount: "عنصر تنظيف محدد",
      floatingReclaimable: "سيتم تحريرها بعد التحقق من الأمان",
      reviewCleanBtn: "مراجعة وتنفيذ التنظيف",
      confirmCleanBtn: "تأكيد التنظيف الآمن الآن",
      cancelConfirmBtn: "تراجع",
      confirmWarning:
        "سيتم التحقق من كل ملف ضد مسارات النظام المحمية وفحص عدم وجود مقبض مفتوح (Active Handle Lock) قبل الحذف، ثم التحقق من إزالته فعلياً.",
      cleanSuccessBanner: "اكتملت عملية التنظيف الآمن وتم التحقق من تحرير المساحة وتوثيقها في SQLite:",
      emptyStateTitle: "اضغط على «فحص عناصر التنظيف الآمن» لإنشاء خطة معاينة",
      emptyStateDesc:
        "لا يحذف SALEEM أي ملف تلقائياً؛ يقوم المحرك أولاً بإنشاء خطة معاينة مفصلة مع استبعاد أي ملفات مفتوحة من البرامج الحالية.",
      zeroCandidatesTitle: "نظامك نظيف تماماً في الفئات المحددة!",
      zeroCandidatesDesc: "لم يتم العثور على ملفات مؤقتة أو كاش غير مقفل يحتاج إلى تنظيف حالياً.",
    },
    duplicates: {
      title: "مكتشف الملفات المكررة الذكي (SHA-256 Content Engine)",
      subtitle:
        "كشف ثلاثي المراحل (الحجم ← بصمة الأطراف 8KB ← SHA-256 الكامل) مع حماية إجبارية لإبقاء نسخة أصلية واحدة على الأقل",
      scanBtn: "فحص التكرارات",
      cancelBtn: "إيقاف الفحص",
      scanningTitle: "جاري مقارنة البصمات الرقمية SHA-256 للملفات المتطابقة...",
      minSizeLabel: "أقل حجم:",
      smartSelectOldest: "الاحتفاظ بالأقدم وتحديد الأحدث للحذف",
      smartSelectNewest: "الاحتفاظ بالأحدث وتحديد الأقدم للحذف",
      clearSelection: "إلغاء جميع التحديدات",
      groupsCount: "مجموعات متطابقة",
      reclaimableSpace: "مساحة قابلة للاسترداد",
      fullHashesVerified: "بصمات SHA-256 كاملة",
      eachFileSize: "حجم النسخة الواحدة",
      copiesBadge: "نسخ متطابقة",
      retainedBadge: "محفوظ بأمان (النسخة الباقية)",
      markedForDeleteBadge: "محدد للإزالة",
      hardlinkBadge: "رابط صلب NTFS Hardlink",
      hardlinkWarning: "هذه النسخ تشير لنفس البيانات الفيزيائية على القرص؛ حذف إحداها لا يحرر مساحة إضافية.",
      keepThisCopy: "الاحتفاظ بهذه النسخة",
      selectForRemoval: "تحديد للإزالة",
      lockedRetainedHint: "مقفلة كنسخة أصلية محفوظة (لا يمكن حذف جميع النسخ)",
      floatingSelectedCount: "ملف مكرر محدد للإزالة",
      floatingReclaimable: "سيتم تحريرها بعد التحقق من SHA-256",
      reviewAndRemoveBtn: "مراجعة وحذف المكرر المحدد",
      confirmRemoveBtn: "تأكيد الحذف النهائي الآن",
      cancelConfirmBtn: "تراجع",
      confirmModalWarning:
        "سيقوم SALEEM بإعادة التحقق من بصمة SHA-256 وحجم كل ملف قبل حذفه لمنع حالات السباق (TOCTOU)، مع ضمان بقاء النسخة المحفوظة في كل مجموعة.",
      removalSuccessBanner: "تم تنفيذ إزالة النسخ المكررة بنجاح وتوثيق العملية في سجل التدقيق:",
      emptyStateTitle: "ابدأ فحص التكرارات لاكتشاف النسخ المتطابقة بالمحتوى",
      emptyStateDesc:
        "لا يعتمد SALEEM على اسم الملف أبداً؛ يتم فحص المحتوى الفعلي عبر خوارزمية SHA-256 مع كشف الروابط الصلبة NTFS Hardlinks.",
      noDuplicatesFoundTitle: "لا توجد ملفات مكررة في هذا المسار!",
      noDuplicatesFoundDesc:
        "جميع الملفات التي تم فحصها فريدة في محتواها ولا توجد نسخ مكررة تهدر المساحة.",
    },
    applications: {
      title: "جرد البرامج والألعاب واكتشاف بقايا الإزالة (SALEEM Inventory)",
      subtitle:
        "فحص سجل Windows ومكتبات الألعاب (Steam / Epic) ومطابقتها مع مجلدات AppData وProgramData لاكتشاف البقايا اليتيمة مع حماية ملفات الحفظ (Saved Games)",
      scanBtn: "فحص البرامج والبقايا اليتيمة",
      cancelBtn: "إيقاف الفحص",
      scanningTitle: "جاري جرد سجل Windows ومكتبات الألعاب وفحص البقايا اليتيمة...",
      tabLeftovers: "بقايا البرامج والألعاب (Orphaned Leftovers)",
      tabInstalled: "التطبيقات والألعاب المثبتة (Installed Inventory)",
      totalApps: "برامج مثبتة (Registry)",
      totalGames: "ألعاب مكتشفة (Steam/Epic)",
      totalLeftovers: "مجلدات بقايا يتيمة",
      highConfidenceBytes: "بقايا عالية الثقة قابلة للاسترداد",
      filterAll: "الكل",
      filterHigh: "ثقة عالية (High)",
      filterMedium: "ثقة متوسطة (Medium)",
      filterLow: "مراجعة مطلوبة (Low)",
      filterSavedProtected: "يحتوي بيانات حفظ محمية",
      selectHighConfidence: "تحديد البقايا عالية الثقة فقط",
      clearSelection: "إلغاء التحديد",
      confidenceHigh: "ثقة عالية (High Confidence)",
      confidenceMedium: "ثقة متوسطة (Medium)",
      confidenceLow: "مراجعة مطلوبة (Low)",
      savedDataProtectedBadge: "محمي افتراضياً: بيانات حفظ / إعدادات",
      lockedFilesBadge: "يحتوي ملفات قيد الاستخدام",
      estimatedSizeBadge: "حجم تقديري (Registry)",
      exactSizeBadge: "حجم دقيق (Manifest)",
      searchAppsPlaceholder: "ابحث باسم البرنامج أو اللعبة أو الناشر...",
      searchLeftoversPlaceholder: "ابحث باسم مجلد البقايا أو المسار...",
      sourceAll: "جميع المصادر",
      sourceRegistry: "تطبيقات Windows",
      sourceSteam: "ألعاب Steam",
      sourceEpic: "ألعاب Epic",
      colAppName: "اسم التطبيق / اللعبة",
      colPublisher: "الناشر والنطاق",
      colVersion: "الإصدار",
      colSource: "المصدر",
      colSize: "الحجم",
      colActions: "إجراءات",
      floatingSelectedCount: "مجلد بقايا محدد للحذف",
      floatingReclaimable: "سيتم تحريرها بعد التحقق من الأمان والأقفال",
      reviewRemoveBtn: "مراجعة وحذف البقايا المحددة",
      confirmRemoveBtn: "تأكيد حذف البقايا النهائي",
      cancelConfirmBtn: "تراجع",
      confirmWarning:
        "سيقوم SALEEM بالتحقق من أن كل مجلد يقع ضمن نطاق AppData/ProgramData المسموح ولا يحتوي على ملفات مقفلة قبل حذفه وتوثيق العملية في SQLite.",
      removeSuccessBanner: "تم حذف مجلدات البقايا المحددة بنجاح وتوثيقها في سجل التدقيق:",
      emptyStateTitle: "اضغط على «فحص البرامج والبقايا اليتيمة» لبدء الجرد الذكي",
      emptyStateDesc:
        "يقارن المحرك جميع البرامج والألعاب المثبتة حالياً مع مجلدات AppData وProgramData لاكتشاف المجلدات اليتيمة مع حماية ملفات حفظ الألعاب (Saved Games) افتراضياً.",
      zeroLeftoversTitle: "لم يتم العثور على مجلدات بقايا يتيمة في هذه الفئة!",
      zeroLeftoversDesc: "جميع المجلدات المفحوصة مرتبطة ببرامج مثبتة حالياً أو محمية ضمن النظام.",
    },
    history: {
      title: "سجل العمليات والتدقيق",
      subtitle: "سجل غير قابل للتلاعب لجميع عمليات الفحص والتغييرات المحفوظة في SQLite",
      empty: "لا توجد سجلات عمليات حتى الآن.",
      colTime: "التوقيت",
      colType: "نوع الحدث",
      colSubject: "المكون",
      colResult: "النتيجة",
      colDetails: "التفاصيل",
    },
    settings: {
      title: "إعدادات التطبيق وحالة المحركات",
      subtitle: "تخصيص لغة الواجهة (RTL/LTR) والمظهر ومراجعة حالة المحركات وضوابط الأمان",
      languageLabel: "لغة الواجهة والاتجاه",
      themeLabel: "مظهر الواجهة",
      safetyPolicyTitle: "ضوابط السلامة غير القابلة للتفاوض (SALEEM Core Policy)",
      safetyPolicyDesc:
        "يعمل SALEEM بصلاحيات المستخدم العادي افتراضياً، ويمنع حذف ملفات النظام الحرجة (WinSxS, DriverStore, System32, System Volume Information)، ولا ينفذ أي حذف دون معاينة وموافقة صريحة.",
    },
    phasePlaceholder: {
      scheduledTitle: "مجدول في المرحلة القادمة وفق خطة التنفيذ",
      scheduledDesc:
        "لا يعرض SALEEM بيانات وهمية أو أزراراً صورية. سيتم تفعيل هذه الوحدة وربطها بمحركها الخلفي واختباراتها في مرحلتها المخصصة وفق ملف MASTER_PROJECT_SPECIFICATION.md.",
    },
  },
  en: {
    appName: "SALEEM",
    appSubtitle: "Disk Intelligence & Static Security for Windows",
    sidebarGroupMain: "INTELLIGENCE",
    sidebarGroupFiles: "FILES & SECURITY",
    nav: {
      overview: "Smart Scan",
      storage: "Space Map",
      large_files: "Large Files",
      duplicates: "Duplicates",
      cleanup: "Safe Cleanup",
      applications: "Apps & Games",
      startup: "System Booster",
      memory: "Memory Engine",
      sentinel: "Sentinel Shield",
      quarantine: "Quarantine",
      history: "Audit Log",
      settings: "Settings",
    },
    header: {
      refresh: "Refresh",
      languageToggle: "العربية (RTL)",
      themeDark: "Dark Mode",
      themeLight: "Light Mode",
      phaseBadge: "Phase 4: App & Leftover Inventory + Crystal Glass UI",
    },
    overview: {
      readyBadge: "ENGINES READY • LIVE OS TELEMETRY",
      scanningBadge: "SCANNING DIRECTORY TREE...",
      heroHeadlineNoScan: "Total active storage across mounted drives",
      heroHeadlineScanned: "Analyzed directory space distribution",
      heroDescNoScan:
        "SALEEM Disk Engine is ready to scan folders in parallel, compute logical vs. allocated cluster sizes, and pinpoint space hogs without freezing the UI.",
      heroDescScanned:
        "Explore the interactive Bubble Map or Treemap, drill down into any subfolder, or inspect the largest files discovered.",
      startScanCta: "Quick Scan",
      exploreMapCta: "Explore Interactive Space Map →",
      spaceMapTitle: "Space map",
      spaceMapOpen: "Open ›",
      spaceMapHint: "Click any sphere to inspect or drill into that path",
      bentoStorageTitle: "Space Analyzer",
      bentoStorageSub: "Parallel tree walker with symlink loop protection & partial-scan tracking",
      bentoStorageBtn: "Explore →",
      bentoLargeTitle: "Large Files",
      bentoLargeSub: "Heaviest files ranked by logical & physical allocated disk size",
      bentoLargeBtn: "Review →",
      bentoSentinelTitle: "Sentinel Shield",
      bentoSentinelSub: "Non-executing static PE, hash & Authenticode signature engine",
      bentoSentinelBtn: "Engine Status →",
      bentoCleanupTitle: "Safe Cleanup",
      bentoCleanupSub: "Hardened policy protecting WinSxS, System32 & DriverStore",
      bentoCleanupBtn: "Policies →",
      enginesTitle: "Native Engine Modules & C-ABI Contract",
      abiVersion: "ABI Version",
      loadMode: "Binding Mode",
      databaseTitle: "SQLite WAL Database & Audit Health",
      schemaVersion: "Schema Version",
      tablesCount: "Active Tables",
      auditEventsCount: "Audit Events",
      activeQuarantineCount: "Quarantined Items",
      usedSpace: "Used",
      freeSpace: "Free",
      totalSpace: "Total",
    },
    analyzer: {
      title: "Storage Analyzer & Interactive Space Map",
      subtitle:
        "Select any drive or folder to analyze logical byte size vs. physical allocated cluster space on disk",
      customPathPlaceholder: "Enter folder or drive path to scan (e.g. D:\\ or C:\\Users)...",
      scanBtn: "Scan Path",
      cancelBtn: "Stop Scan Immediately",
      scanningTitle: "SALEEM Disk Engine is scanning directory hierarchy...",
      filesSeen: "files scanned",
      foldersSeen: "folders",
      logicalTotal: "Logical Size",
      allocatedTotal: "Allocated on Disk",
      partialScanTitle: "Partial Scan Result",
      partialScanDesc:
        "Some subdirectories could not be read due to Windows permissions (Access Denied). Totals reflect accessible items only.",
      reparseSkipped: "Reparse points / junctions skipped to prevent loops",
      viewBubbles: "Bubble Map",
      viewTreemap: "Treemap Grid",
      spaceMapSection: "Visual Directory Distribution (Click any item to drill down)",
      categoriesSection: "Breakdown by File Category",
      extensionsSection: "Top Extensions by Space",
      foldersSection: "Discovered Subdirectories",
      searchPlaceholder: "Filter folders by name or path...",
      sortByLogical: "Sort: Logical Size",
      sortByAllocated: "Sort: Allocated Size",
      sortByFiles: "Sort: File Count",
      sortByName: "Sort: Name",
      openExplorer: "Reveal in Explorer",
      copyPath: "Copy Path",
      copied: "Copied!",
      drillDown: "Scan Inside",
      upOneLevel: "Parent Folder ↑",
      emptyStateTitle: "Select a drive or preset folder to begin analysis",
      emptyStateDesc:
        "Click a quick preset chip above or enter any path to build the interactive Space Map and rank the largest folders and files.",
    },
    largeFiles: {
      title: "Deep Large Files Intelligence & Safe Removal",
      subtitle:
        "Deep extraction of the heaviest files on disk with specialized category intelligence and user-controlled verified deletion",
      allCategories: "All Categories",
      allSizes: "All Sizes",
      searchPlaceholder: "Filter by filename, extension, or folder path...",
      colSelect: "Select",
      colName: "File Name & Path",
      colCategory: "Category & Ext",
      colLogicalSize: "Logical Size",
      colAllocatedSize: "Allocated Size",
      colModified: "Modified Date",
      colActions: "Actions",
      noScanYet:
        "Run a scan from Smart Scan or Space Map first to extract the largest files on disk.",
      selectedCount: "large files selected for removal",
      reviewDeleteBtn: "Review & Delete Selected",
      confirmDeleteBtn: "Confirm Verified Deletion",
      confirmWarning:
        "SALEEM will verify each selected file against Protected System Paths, check for active process locks, and verify exact byte size before deletion.",
      deleteSuccessBanner: "Verified large file removal completed and logged to SQLite audit trail:",
    },
    cleanup: {
      title: "Rule-Based Safe Cleanup Engine (9-Stage Pipeline)",
      subtitle:
        "Scan → Analyze → Plan → Preview → Confirm → Validate → Execute → Verify → Report with active open-handle lock protection",
      scanBtn: "Scan Safe Cleanup Targets",
      cancelBtn: "Stop Scan",
      scanningTitle: "SALEEM Cleanup Engine is inspecting user temp, cache & diagnostic scopes...",
      pipelineBadge: "9-STAGE PIPELINE • ZERO AUTO-DELETE",
      rulesSectionTitle: "Declarative Cleanup Rules (Toggle categories to include)",
      candidatesSectionTitle: "Cleanup Plan Preview (Select items to clean)",
      selectAllSafe: "Select All Safe (Low Risk)",
      clearSelection: "Clear Selection",
      totalCandidates: "Safe Candidates",
      lockedSkipped: "Locked Files Skipped",
      reclaimableBytes: "Reclaimable Space",
      riskLow: "Safe (Low Risk)",
      riskMedium: "Review (Medium)",
      floatingSelectedCount: "cleanup items selected",
      floatingReclaimable: "to reclaim with post-delete verification",
      reviewCleanBtn: "Review & Clean Selected",
      confirmCleanBtn: "Confirm Safe Cleanup Now",
      cancelConfirmBtn: "Cancel",
      confirmWarning:
        "Every file will be re-validated against the Protected System Path Policy and tested for exclusive file handle locks before removal.",
      cleanSuccessBanner: "Safe cleanup completed, verified on disk, and recorded in SQLite:",
      emptyStateTitle: "Click 'Scan Safe Cleanup Targets' to generate a preview plan",
      emptyStateDesc:
        "SALEEM never deletes files automatically. It first builds an explainable preview plan while safely skipping any files currently locked by running apps.",
      zeroCandidatesTitle: "All Selected Scopes Are Clean!",
      zeroCandidatesDesc: "No unlocked temporary or cache files were found matching the active rules.",
    },
    duplicates: {
      title: "Smart Duplicate Finder (3-Stage SHA-256 Engine)",
      subtitle:
        "Exact content matching (Size → 8KB Head/Tail Hash → Full SHA-256) with mandatory retained-copy protection & NTFS hardlink awareness",
      scanBtn: "Scan Duplicates",
      cancelBtn: "Stop Scan",
      scanningTitle: "Computing 3-stage SHA-256 cryptographic digests...",
      minSizeLabel: "Min Size:",
      smartSelectOldest: "Keep Oldest • Select Newer Copies",
      smartSelectNewest: "Keep Newest • Select Older Copies",
      clearSelection: "Clear Selection",
      groupsCount: "Duplicate Groups",
      reclaimableSpace: "Reclaimable Space",
      fullHashesVerified: "Full SHA-256 Verified",
      eachFileSize: "Size per copy",
      copiesBadge: "identical copies",
      retainedBadge: "PROTECTED (RETAINED COPY)",
      markedForDeleteBadge: "MARKED FOR REMOVAL",
      hardlinkBadge: "NTFS Hard Link",
      hardlinkWarning: "Hard-linked siblings share physical NTFS clusters; deleting one does not free extra disk space.",
      keepThisCopy: "Keep This Copy",
      selectForRemoval: "Select for Removal",
      lockedRetainedHint: "Locked as retained copy (cannot delete all copies)",
      floatingSelectedCount: "duplicate files selected",
      floatingReclaimable: "to reclaim after TOCTOU SHA-256 check",
      reviewAndRemoveBtn: "Review & Remove Selected",
      confirmRemoveBtn: "Confirm Verified Deletion",
      cancelConfirmBtn: "Cancel",
      confirmModalWarning:
        "SALEEM will re-verify each file's size and full SHA-256 hash immediately before deletion (TOCTOU protection) and guarantee the retained copy remains untouched.",
      removalSuccessBanner: "Verified duplicate removal completed and logged to SQLite audit trail:",
      emptyStateTitle: "Run a Duplicate Scan to discover identical files by content",
      emptyStateDesc:
        "SALEEM never matches by filename. Files are grouped strictly by verified SHA-256 content hashes with automatic NTFS hardlink inspection.",
      noDuplicatesFoundTitle: "Zero Duplicate Files Found!",
      noDuplicatesFoundDesc:
        "Every file examined in this directory above the minimum size threshold has unique content.",
    },
    applications: {
      title: "Application, Game & Orphaned Leftover Inventory",
      subtitle:
        "Cross-references Windows Registry Uninstall hives & Steam/Epic game manifests against AppData & ProgramData with automatic Saved Games protection",
      scanBtn: "Scan Apps & Orphaned Leftovers",
      cancelBtn: "Stop Scan",
      scanningTitle: "Scanning Windows Registry hives, Game Libraries & AppData leftovers...",
      tabLeftovers: "Orphaned Leftovers",
      tabInstalled: "Installed Apps & Games",
      totalApps: "Installed Apps (Registry)",
      totalGames: "Installed Games (Steam/Epic)",
      totalLeftovers: "Orphaned Leftover Folders",
      highConfidenceBytes: "High-Confidence Reclaimable",
      filterAll: "All",
      filterHigh: "High Confidence",
      filterMedium: "Medium Confidence",
      filterLow: "Review Required (Low)",
      filterSavedProtected: "Saved Games Protected",
      selectHighConfidence: "Select High Confidence Only",
      clearSelection: "Clear Selection",
      confidenceHigh: "High Confidence",
      confidenceMedium: "Medium Confidence",
      confidenceLow: "Review Required (Low)",
      savedDataProtectedBadge: "Protected by Default: Saved Games / User Data",
      lockedFilesBadge: "Contains Locked Active Files",
      estimatedSizeBadge: "Estimated (Registry)",
      exactSizeBadge: "Exact (Manifest)",
      searchAppsPlaceholder: "Filter by app/game name or publisher...",
      searchLeftoversPlaceholder: "Filter by leftover folder name or path...",
      sourceAll: "All Sources",
      sourceRegistry: "Windows Apps",
      sourceSteam: "Steam Games",
      sourceEpic: "Epic Games",
      colAppName: "Application / Game Name",
      colPublisher: "Publisher & Scope",
      colVersion: "Version",
      colSource: "Source",
      colSize: "Size",
      colActions: "Actions",
      floatingSelectedCount: "leftover folders selected",
      floatingReclaimable: "to reclaim after scope & lock verification",
      reviewRemoveBtn: "Review & Remove Leftovers",
      confirmRemoveBtn: "Confirm Verified Leftover Removal",
      cancelConfirmBtn: "Cancel",
      confirmWarning:
        "SALEEM will verify each selected folder resides strictly inside allowed AppData/ProgramData scopes and contains no locked files before removal.",
      removeSuccessBanner: "Verified leftover folders removed and recorded in SQLite audit trail:",
      emptyStateTitle: "Click 'Scan Apps & Orphaned Leftovers' to start discovery",
      emptyStateDesc:
        "SALEEM compares all installed Windows applications and Steam/Epic games against AppData and ProgramData folders while protecting Saved Games by default.",
      zeroLeftoversTitle: "No Orphaned Leftover Folders Found in This Filter!",
      zeroLeftoversDesc: "All inspected folders match active installed applications or protected system components.",
    },
    history: {
      title: "Activity & Audit History",
      subtitle: "Transactional audit trail recorded in local SQLite WAL database",
      empty: "No audit events recorded yet.",
      colTime: "Timestamp",
      colType: "Event Type",
      colSubject: "Component",
      colResult: "Result",
      colDetails: "Details",
    },
    settings: {
      title: "Settings, Engines & Safety Policies",
      subtitle: "Configure interface language (RTL/LTR), theme mode, and inspect native engine bindings",
      languageLabel: "Interface Language & Direction",
      themeLabel: "Appearance Theme",
      safetyPolicyTitle: "Non-Negotiable Safety Boundaries (SALEEM Core Policy)",
      safetyPolicyDesc:
        "SALEEM runs with standard user privileges by default, strictly blocks modification of critical Windows paths (WinSxS, DriverStore, System32, System Volume Information), and requires preview and confirmation before any file action.",
    },
    phasePlaceholder: {
      scheduledTitle: "Scheduled for Next Implementation Phase",
      scheduledDesc:
        "SALEEM never displays fake metrics or non-functional buttons. This module will be activated with its full Rust backend engine and test suite in its designated phase per MASTER_PROJECT_SPECIFICATION.md.",
    },
  },
};
