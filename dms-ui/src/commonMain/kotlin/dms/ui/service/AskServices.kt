package dms.ui.service

import dms.application.ask.AskLibrary
import dms.application.ask.IndexLibrary
import dms.application.ask.RelatedDocuments

/** Asking the library: building the embedding index, answering questions, and related documents. */
class AskServices(
    val indexLibrary: IndexLibrary,
    val askLibrary: AskLibrary,
    val relatedDocuments: RelatedDocuments,
)
