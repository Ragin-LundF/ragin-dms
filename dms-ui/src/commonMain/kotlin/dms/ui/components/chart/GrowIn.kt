package dms.ui.components.chart

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue

/** Animates from 0 to [target] when first shown, and to every later target from where it is. */
@Composable
fun growIn(target: Float): Float {
    var shown by remember { mutableStateOf(value = false) }
    LaunchedEffect(key1 = Unit) {
        shown = true
    }
    val value by animateFloatAsState(
        targetValue = if (shown) target else 0f,
        animationSpec = tween(durationMillis = 700),
        label = "grow-in",
    )
    return value
}
