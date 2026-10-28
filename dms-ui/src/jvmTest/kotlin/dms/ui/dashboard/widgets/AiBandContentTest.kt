package dms.ui.dashboard.widgets

import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.widgets.model.AiBandModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class AiBandContentTest {

    @Test
    fun `there is no band while the AI has nothing to tag or classify`() {
        val band = AiBandContent.of(state = DashboardUiStateModel(tagCandidates = 0, classifyCandidates = 0))

        assertNull(actual = band)
    }

    @Test
    fun `there is no band without AI set up, whatever there is to do`() {
        val state = DashboardUiStateModel(tagCandidates = 5, classifyCandidates = 2, aiEnabled = false)

        assertNull(actual = AiBandContent.of(state = state))
    }

    @Test
    fun `the band names both counts and offers both runs`() {
        val band = AiBandContent.of(state = DashboardUiStateModel(tagCandidates = 5, classifyCandidates = 2))

        assertEquals(
            expected = AiBandModel(
                headline = "5 documents have no tags yet, 2 have no category",
                classifyLabel = "Classify 2",
                tagLabel = "Tag 5 with AI",
                busyReason = null,
            ),
            actual = band,
        )
    }

    @Test
    fun `a run without work has no button and is left out of the headline`() {
        val tagsOnly = AiBandContent.of(state = DashboardUiStateModel(tagCandidates = 1, classifyCandidates = 0))
        val categoriesOnly = AiBandContent.of(state = DashboardUiStateModel(tagCandidates = 0, classifyCandidates = 3))

        assertEquals(expected = "1 document has no tags yet", actual = tagsOnly?.headline)
        assertNull(actual = tagsOnly?.classifyLabel)
        assertEquals(expected = "3 documents have no category", actual = categoriesOnly?.headline)
        assertNull(actual = categoriesOnly?.tagLabel)
        assertEquals(expected = "Classify 3", actual = categoriesOnly?.classifyLabel)
    }

    @Test
    fun `a running AI batch keeps both buttons waiting and says why`() {
        val state = DashboardUiStateModel(tagCandidates = 2, classifyCandidates = 1, aiBusy = "Tagging 2 documents")

        val band = AiBandContent.of(state = state)

        assertEquals(expected = "Wait until \"Tagging 2 documents\" finishes", actual = band?.busyReason)
    }
}
