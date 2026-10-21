package dms.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AddBox
import androidx.compose.material.icons.rounded.IndeterminateCheckBox
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp

/**
 * The boxed + or - that unfolds or folds a tree entry or a section. With [expanded] `null` the entry
 * has nothing to fold, and the space stays empty so names line up.
 */
@Composable
fun FoldToggle(expanded: Boolean?, onToggle: () -> Unit) {
    val modifier = Modifier.size(size = 18.dp)
    if (expanded == null) {
        Spacer(modifier = modifier)
        return
    }
    Icon(
        imageVector = if (expanded) Icons.Rounded.IndeterminateCheckBox else Icons.Rounded.AddBox,
        contentDescription = if (expanded) "Fold" else "Unfold",
        tint = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = modifier.clip(shape = MaterialTheme.shapes.extraSmall).clickable(onClick = onToggle),
    )
}
