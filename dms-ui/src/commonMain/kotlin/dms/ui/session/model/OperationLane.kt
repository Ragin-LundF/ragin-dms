package dms.ui.session.model

/**
 * Operations that must not run at the same time share a lane; operations in different lanes run
 * side by side. [LIBRARY] holds everything that changes files or the catalog, so a scan never sees
 * a move half done. [AI] holds the AI's runs, which only add suggestions and use one AI server.
 */
enum class OperationLane {
    LIBRARY,
    AI,
}
