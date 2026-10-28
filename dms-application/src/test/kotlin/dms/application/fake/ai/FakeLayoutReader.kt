package dms.application.fake.ai

import dms.application.port.ai.PageLayoutReader
import dms.application.port.ai.model.PageImageModel
import dms.application.port.ai.model.layout.LayoutOutcome
import dms.application.port.ai.model.layout.LineOutcome
import dms.domain.rebuild.model.PageLayoutModel

/**
 * Answers with the scripted [layouts] by page number, an empty page otherwise, and with [lines] by the
 * crop's image text (its box). [beforeLayout] runs before each answer, so a test can hold a page.
 */
class FakeLayoutReader : PageLayoutReader {

    val layouts: MutableMap<Int, LayoutOutcome> = mutableMapOf()
    val lines: MutableMap<String, LineOutcome> = mutableMapOf()
    var beforeLayout: suspend (Int) -> Unit = {}

    override suspend fun readLayout(page: PageImageModel): LayoutOutcome {
        val number = page.bytes.decodeToString().removePrefix(prefix = "page ").toInt()
        beforeLayout(number)
        return layouts[number] ?: LayoutOutcome.Read(layout = EMPTY)
    }

    override suspend fun readLine(line: PageImageModel): LineOutcome {
        return lines[line.bytes.decodeToString()] ?: LineOutcome.Read(text = "")
    }

    private companion object {
        val EMPTY = PageLayoutModel(lines = emptyList(), graphics = emptyList())
    }
}
