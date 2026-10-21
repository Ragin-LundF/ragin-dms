package dms.application.port.storage.model

/** The result of a storage operation. Expected failures are values, not exceptions. */
sealed interface StorageResult<out T> {

    data class Success<T>(val value: T) : StorageResult<T>

    data class Failure(val reason: StorageFailure, val detail: String? = null) : StorageResult<Nothing>
}
