package dms.pdf.ocr

import dms.application.port.pdf.OcrTool
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.pdf.io.PdfFiles
import dms.pdf.io.PdfLocations
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.nio.file.Files

/**
 * "Precise OCR" with `ocrmypdf`, if it is installed and on the PATH; nothing is bundled. It writes a
 * searchable copy to a temporary file, which is then moved to the target.
 */
class OcrMyPdf(
    private val locations: PdfLocations,
    private val runner: CommandRunner = ProcessRunner(),
    private val program: String = PROGRAM,
) : OcrTool {

    override suspend fun available(): Boolean {
        return withContext(context = Dispatchers.IO) { runner.run(command = listOf(program, "--version")) == 0 }
    }

    override suspend fun ocr(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        return PdfFiles.io {
            val input = locations.existing(folder, source)
                ?: return@io StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
            val output = locations.target(folder, target)
                ?: return@io StorageResult.Failure(reason = StorageFailure.FOLDER_UNREACHABLE)
            val temp = PdfFiles.tempFor(target = output)
            try {
                val exit = runner.run(
                    command = listOf(program, "--skip-text", "--quiet", input.toString(), temp.toString()),
                )
                if (exit != 0 || !Files.exists(temp)) {
                    StorageResult.Failure(reason = StorageFailure.IO_ERROR, detail = "$program ended with $exit")
                } else {
                    PdfFiles.place(temp = temp, target = output)
                }
            } finally {
                Files.deleteIfExists(temp)
            }
        }
    }

    private companion object {
        const val PROGRAM = "ocrmypdf"
    }
}
