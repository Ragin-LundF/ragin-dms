package dms.application.port.pdf.model

import dms.domain.rebuild.model.PageLayoutModel

/** What the rebuild writes for one page. */
sealed interface PageInput {

    /** A page the AI read: rebuilt from [layout] on the straightened [prepared] page. */
    data class Read(val prepared: PreparedPageModel, val layout: PageLayoutModel) : PageInput

    /** A page the AI could not read: page [pageIndex] of the original stays as it is. */
    data class Failed(val pageIndex: Int) : PageInput
}
