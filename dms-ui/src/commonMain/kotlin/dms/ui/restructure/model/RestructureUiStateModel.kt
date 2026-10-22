package dms.ui.restructure.model

import dms.domain.restructure.FileNaming

/**
 * The restructure assistant: the [scopes] to choose from, the [template] being written, and the plan
 * as [groups] of moves with what stays ([unchanged]), what is left out ([skipped] by reason) and the
 * names that got a suffix ([renamed]). [history] lists applied restructures that can be reversed.
 */
data class RestructureUiStateModel(
    val scopes: List<ScopeOptionModel> = emptyList(),
    val scopeIndex: Int = 0,
    val template: String = "",
    val naming: FileNaming = FileNaming.KEEP,
    val problem: String? = null,
    val planned: Boolean = false,
    val groups: List<MoveGroupModel> = emptyList(),
    val unchanged: Int = 0,
    val skipped: List<String> = emptyList(),
    val renamed: List<String> = emptyList(),
    val history: List<String> = emptyList(),
    val savedTemplate: String? = null,
    val busy: String? = null,
) {
    val includedMoves: Int
        get() {
            return groups.filter { it.included }.sumOf { it.count }
        }
}
