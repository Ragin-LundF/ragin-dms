package dms.ui.library

import androidx.compose.runtime.Composable
import dms.ui.assistant.ChatDialog
import dms.ui.assistant.SummaryDialog
import dms.ui.components.dialog.DialogHost
import dms.ui.document.InspectorViewModel
import dms.ui.review.ReviewDialog

/** The dialogs the library and its inspector can open: confirmations, the review, the summary and the chat. */
@Composable
fun LibraryDialogs(viewModel: LibraryViewModel, inspector: InspectorViewModel) {
    DialogHost(controller = viewModel.dialogs)
    DialogHost(controller = inspector.dialogs)
    ReviewDialog(controller = inspector.review, onOpenDocument = inspector::open)
    SummaryDialog(controller = inspector.summary, onAsk = { inspector.chat.open(document = it) })
    ChatDialog(controller = inspector.chat)
}
