package dms.domain.settings

import dms.domain.library.FileNameViolation
import dms.domain.library.model.RelativePathModel
import dms.domain.settings.model.InboxSettingsModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class InboxSettingsTest {

    @Test
    fun `the inbox path must be a valid folder below the root`() {
        assertNull(actual = InboxSettingsModel.pathViolation(path = RelativePathModel.of(raw = "Inbox/Scanner")))
        assertEquals(
            expected = FileNameViolation.EMPTY,
            actual = InboxSettingsModel.pathViolation(path = RelativePathModel.ROOT),
        )
        assertEquals(
            expected = FileNameViolation.LEADING_DOT_OR_SPACE,
            actual = InboxSettingsModel.pathViolation(path = RelativePathModel.of(raw = ".dms-text/x")),
        )
        assertEquals(
            expected = FileNameViolation.RESERVED_NAME,
            actual = InboxSettingsModel.pathViolation(path = RelativePathModel.of(raw = "CON")),
        )
    }

    @Test
    fun `only documents below the inbox count, and only while it is on`() {
        val inbox = InboxSettingsModel(enabled = true)

        assertTrue(actual = inbox.contains(candidate = RelativePathModel.of(raw = "Inbox/a.pdf")))
        assertFalse(actual = inbox.contains(candidate = RelativePathModel.of(raw = "Inboxes/a.pdf")))
        assertFalse(
            actual = inbox.copy(enabled = false).contains(candidate = RelativePathModel.of(raw = "Inbox/a.pdf")),
        )
    }
}
