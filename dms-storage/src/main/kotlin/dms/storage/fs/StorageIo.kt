package dms.storage.fs

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import tools.jackson.core.JacksonException
import java.io.IOException
import java.io.UncheckedIOException

/** Runs file system work on the IO dispatcher and turns I/O exceptions into storage failures. */
object StorageIo {

    suspend fun <T> run(block: () -> StorageResult<T>): StorageResult<T> {
        return withContext(context = Dispatchers.IO) {
            catching(block = block)
        }
    }

    fun <T> catching(block: () -> StorageResult<T>): StorageResult<T> {
        return try {
            block()
        } catch (exception: IOException) {
            StorageResult.Failure(reason = StorageFailure.IO_ERROR, detail = exception.message)
        } catch (exception: UncheckedIOException) {
            StorageResult.Failure(reason = StorageFailure.IO_ERROR, detail = exception.message)
        } catch (exception: JacksonException) {
            StorageResult.Failure(reason = StorageFailure.UNSUPPORTED_FORMAT, detail = exception.originalMessage)
        }
    }

    fun <T> failure(reason: StorageFailure, detail: String? = null): StorageResult<T> {
        return StorageResult.Failure(reason = reason, detail = detail)
    }
}
