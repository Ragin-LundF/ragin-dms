package dms.ui.session

import dms.application.port.system.Clock
import dms.ui.session.model.LogEntry
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

/**
 * What the app did in this session, newest last, for the log panel. Keeps the last [capacity] lines;
 * nothing is written to disk. Safe to call from any thread.
 */
class SessionLog(private val clock: Clock, private val capacity: Int = DEFAULT_CAPACITY) {

    private val mutableEntries = MutableStateFlow(value = emptyList<LogEntry>())

    val entries: StateFlow<List<LogEntry>> = mutableEntries.asStateFlow()

    fun info(text: String) {
        add(entry = LogEntry(at = clock.now(), text = text))
    }

    fun error(text: String) {
        add(entry = LogEntry(at = clock.now(), text = text, isError = true))
    }

    fun clear() {
        mutableEntries.value = emptyList()
    }

    private fun add(entry: LogEntry) {
        mutableEntries.update { entries -> (entries + entry).takeLast(n = capacity) }
    }

    private companion object {
        const val DEFAULT_CAPACITY = 2_000
    }
}
