package dms.ui.settings.ai

import androidx.compose.runtime.Composable

/** The switch that makes the AI read PDFs as rendered pages even when they carry text. */
@Composable
fun AiVisionSwitch(checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
    SettingSwitch(
        title = "Read PDFs as images",
        text = "Sends rendered pages even when a PDF has text. Helps with poor scanner text; " +
            "needs a vision model and is slower.",
        checked = checked,
        onCheckedChange = onCheckedChange,
    )
}
