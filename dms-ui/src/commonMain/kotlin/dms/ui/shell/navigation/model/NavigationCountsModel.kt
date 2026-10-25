package dms.ui.shell.navigation.model

/** The numbers beside the navigation entries: documents waiting in the inbox and open tasks. */
data class NavigationCountsModel(val inbox: Int = 0, val tasks: Int = 0)
