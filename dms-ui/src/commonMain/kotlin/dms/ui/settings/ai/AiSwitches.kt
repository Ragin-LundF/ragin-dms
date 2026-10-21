package dms.ui.settings.ai

import androidx.compose.runtime.Composable
import dms.ui.settings.ai.model.AiDraft

/** The switches of the AI settings: how PDFs are read, and whether a review takes the suggested file name. */
@Composable
fun AiSwitches(draft: AiDraft, onChange: (AiDraft) -> Unit) {
    AiVisionSwitch(
        checked = draft.preferVisionForPdfs,
        onCheckedChange = { onChange(draft.copy(preferVisionForPdfs = it)) },
    )
    SettingSwitch(
        title = "Rename to the suggested file name",
        text = "Checks the suggested file name in a review, so approving renames the file. " +
            "When off, the name is still suggested and you check it if it is better.",
        checked = draft.acceptSuggestedFileNames,
        onCheckedChange = { onChange(draft.copy(acceptSuggestedFileNames = it)) },
    )
}
