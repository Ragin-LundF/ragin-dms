package dms.ui.system

import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.window.WindowExceptionHandler
import androidx.compose.ui.window.WindowExceptionHandlerFactory
import java.awt.Window
import java.awt.event.WindowEvent
import javax.swing.JOptionPane
import javax.swing.SwingUtilities

/**
 * Like Compose's default handler, it shows the error and closes the window, but with a message a
 * person can act on, from [ErrorTexts].
 */
@OptIn(ExperimentalComposeUiApi::class)
object AppErrorHandler : WindowExceptionHandlerFactory {

    override fun exceptionHandler(window: Window): WindowExceptionHandler {
        return WindowExceptionHandler { throwable ->
            SwingUtilities.invokeLater {
                // A window that failed while opening cannot be the dialog's parent.
                JOptionPane.showMessageDialog(
                    window.takeIf { it.isDisplayable },
                    ErrorTexts.messageOf(throwable = throwable),
                    ErrorTexts.TITLE,
                    JOptionPane.ERROR_MESSAGE,
                )
                window.dispatchEvent(WindowEvent(window, WindowEvent.WINDOW_CLOSING))
            }
            throw throwable
        }
    }
}
