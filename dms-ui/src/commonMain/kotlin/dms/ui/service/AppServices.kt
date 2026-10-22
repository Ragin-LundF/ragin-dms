package dms.ui.service

/** Every use case the UI calls, built by the composition root. */
class AppServices(
    val library: LibraryServices,
    val documents: DocumentServices,
    val sync: SyncServices,
    val ai: AiServices,
    val organize: OrganizeServices,
    val pdf: PdfServices,
    val ask: AskServices,
)
