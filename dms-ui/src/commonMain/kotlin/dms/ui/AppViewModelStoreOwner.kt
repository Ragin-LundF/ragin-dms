package dms.ui

import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.ViewModelStoreOwner

/** Owns the view models of the main window; cleared when the window's composition ends. */
class AppViewModelStoreOwner : ViewModelStoreOwner {

    override val viewModelStore: ViewModelStore = ViewModelStore()
}
