package dms.ui.library.search

import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ManageSearch
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.library.search.model.ContentSearchState
import dms.ui.theme.Spacing

/** Switches searching the text of the documents on or off; spins while a search runs. */
@Composable
fun ContentSearchChip(state: ContentSearchState, onEnable: (Boolean) -> Unit) {
    FilterChip(
        selected = state.enabled,
        onClick = { onEnable(!state.enabled) },
        label = { Text(text = "Search in text") },
        leadingIcon = {
            if (state.searching) {
                CircularProgressIndicator(modifier = Modifier.size(size = Spacing.icon), strokeWidth = 2.dp)
            } else {
                Icon(
                    imageVector = Icons.AutoMirrored.Rounded.ManageSearch,
                    contentDescription = null,
                    modifier = Modifier.size(size = Spacing.icon),
                )
            }
        },
    )
}
