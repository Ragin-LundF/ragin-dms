package dms.ui.shell

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import dms.ui.session.model.LogEntry
import dms.ui.theme.Spacing
import java.time.ZoneId
import java.time.format.DateTimeFormatter

/** What the app did in this session, newest at the bottom; follows new lines as they come. */
@Composable
fun LogPanel(entries: List<LogEntry>, onClear: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val listState = rememberLazyListState()
    val time = remember { DateTimeFormatter.ofPattern("HH:mm:ss").withZone(ZoneId.systemDefault()) }
    LaunchedEffect(key1 = entries.lastOrNull()) {
        if (entries.isNotEmpty()) {
            listState.scrollToItem(index = entries.lastIndex)
        }
    }
    Surface(
        color = colors.surfaceContainerLowest,
        modifier = Modifier.fillMaxWidth().height(height = Spacing.logHeight),
    ) {
        Column {
            HorizontalDivider(color = colors.outlineVariant)
            Row(
                modifier = Modifier.fillMaxWidth().padding(horizontal = Spacing.large),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(text = "Log of this session", style = MaterialTheme.typography.labelLarge)
                TextButton(onClick = onClear, enabled = entries.isNotEmpty()) { Text(text = "Clear") }
            }
            SelectionContainer {
                LazyColumn(
                    state = listState,
                    modifier = Modifier.fillMaxWidth().padding(horizontal = Spacing.large),
                ) {
                    items(items = entries) { entry ->
                        Text(
                            text = "${time.format(entry.at)}  ${entry.text}",
                            style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                            color = if (entry.isError) colors.error else colors.onSurfaceVariant,
                            maxLines = 1,
                            softWrap = false,
                        )
                    }
                }
            }
        }
    }
}
