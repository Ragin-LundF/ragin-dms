package dms.storage.support

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import kotlin.test.fail

/** Unwraps storage results in assertions. */
object StorageResults {

    fun <T> valueOf(result: StorageResult<T>): T {
        return when (result) {
            is StorageResult.Success -> result.value
            is StorageResult.Failure -> fail(message = "Expected success, got $result")
        }
    }

    fun failureOf(result: StorageResult<*>): StorageFailure? {
        return (result as? StorageResult.Failure)?.reason
    }
}
