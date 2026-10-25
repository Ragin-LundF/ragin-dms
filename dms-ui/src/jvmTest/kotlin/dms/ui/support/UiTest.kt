package dms.ui.support

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain

/** Runs a view model test against a fresh [TestApp], with the main dispatcher of the view models replaced. */
object UiTest {

    @OptIn(ExperimentalCoroutinesApi::class)
    fun run(block: suspend TestScope.(TestApp) -> Unit) {
        Dispatchers.setMain(dispatcher = UnconfinedTestDispatcher())
        try {
            runTest {
                val app = TestApp(scope = backgroundScope)
                try {
                    block(app)
                } finally {
                    // Background work still reading the app's files must end before the files are deleted.
                    backgroundScope.coroutineContext.job.cancelAndJoin()
                    app.close()
                }
            }
        } finally {
            Dispatchers.resetMain()
        }
    }
}
