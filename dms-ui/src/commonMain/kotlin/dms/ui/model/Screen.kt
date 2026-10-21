package dms.ui.model

/** The main screens, in the order of the navigation rail. */
enum class Screen(val label: String, val symbol: String) {
    DASHBOARD(label = "Dashboard", symbol = "⌂"),
    LIBRARY(label = "Library", symbol = "☰"),
    SETTINGS(label = "Settings", symbol = "⚙"),
}
