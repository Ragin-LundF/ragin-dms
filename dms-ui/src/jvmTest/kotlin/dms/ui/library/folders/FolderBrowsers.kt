package dms.ui.library.folders

import dms.ui.components.dialog.DialogController
import dms.ui.library.state.DiskFolders
import dms.ui.support.TestApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers

/** Folder browsers on a [TestApp] whose state follows at once, as on the view models' main dispatcher. */
object FolderBrowsers {

    fun of(app: TestApp, scope: CoroutineScope, dialogs: DialogController = DialogController()): FolderBrowser {
        val main = CoroutineScope(context = scope.coroutineContext + Dispatchers.Main)
        return FolderBrowser(
            services = app.services,
            session = app.session,
            dialogs = dialogs,
            disk = DiskFolders(services = app.services, session = app.session, scope = main),
            scope = main,
        )
    }
}
