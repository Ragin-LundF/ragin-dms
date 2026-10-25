package dms.ui.library

import androidx.compose.runtime.Composable
import dms.ui.assistant.ChatDialog
import dms.ui.assistant.SummaryDialog
import dms.ui.components.dialog.DialogHost
import dms.ui.document.QuickLookViewModel
import dms.ui.export.ExportDialog
import dms.ui.library.picker.FolderPickerDialog
import dms.ui.pdf.MergeDialog
import dms.ui.pdf.PageEditorDialog
import dms.ui.review.ReviewDialog

/**
 * The dialogs the library and its quickLookViewModel can open: confirmations, the folder picker of the bulk
 * bar's "Move", review, summary, chat and export.
 */
@Composable
fun LibraryDialogs(viewModel: LibraryViewModel, quickLookViewModel: QuickLookViewModel) {
    DialogHost(controller = viewModel.dialogs)
    DialogHost(controller = quickLookViewModel.dialogs)
    FolderPickerDialog(picker = viewModel.bulk.picker, foldersOf = viewModel.bulk::pickerFolders)
    ReviewDialog(controller = quickLookViewModel.review, onOpenDocument = quickLookViewModel::open)
    SummaryDialog(controller = quickLookViewModel.summary, onAsk = { quickLookViewModel.chat.open(document = it) })
    ChatDialog(controller = quickLookViewModel.chat)
    ExportDialog(controller = viewModel.export)
    PageEditorDialog(controller = quickLookViewModel.pages)
    MergeDialog(actions = quickLookViewModel.pdf)
}
