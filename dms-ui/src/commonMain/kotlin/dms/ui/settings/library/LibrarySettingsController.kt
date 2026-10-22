package dms.ui.settings.library

import dms.domain.field.FieldPresets
import dms.domain.field.FieldRole
import dms.domain.field.FieldType
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.settings.model.InboxSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel
import dms.ui.settings.library.model.LibrarySettingsStateModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn

/**
 * The settings that belong to the library rather than to this device: they live in the catalog and
 * travel to every folder and device. The settings view model owns it; every change runs through the
 * session as one catalog change.
 */
class LibrarySettingsController(
    private val services: AppServices,
    private val session: LibrarySession,
    scope: CoroutineScope,
) {

    val state: StateFlow<LibrarySettingsStateModel> = combine(
        flow = services.library.catalog,
        flow2 = session.state,
    ) { catalog, sessionState ->
        LibrarySettingsStateModel(settings = catalog?.settings, busy = sessionState.busy)
    }.stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = LibrarySettingsStateModel())

    /** Adds a custom field named [label]; its key is derived from the label and must be new. */
    fun addField(label: String, type: FieldType, role: FieldRole?) {
        val key = FieldKeyModel.fromLabel(label = label)
        val settings = state.value.settings ?: return
        if (key == null || settings.field(key = key) != null) {
            session.report(message = UiMessageModel(text = "Choose another name for the field", isError = true))
            return
        }
        val definition = FieldDefinitionModel(
            key = key,
            label = label.trim(),
            type = type,
            role = role.takeIf { type == FieldType.DATE },
        )
        change(subject = "Field ${definition.label}", done = "Added the field ${definition.label}") {
            it.withField(definition = definition)
        }
    }

    /** Removes a field definition; the values documents have keep their place in the catalog file. */
    fun removeField(key: FieldKeyModel) {
        change(subject = "Field $key", done = "Removed the field") { it.withoutField(key = key) }
    }

    fun addStandardFields() {
        change(subject = "Standard fields", done = "Added the standard fields") { settings ->
            FieldPresets.ALL.fold(initial = settings) { current, preset ->
                if (current.field(key = preset.key) == null) current.withField(definition = preset) else current
            }
        }
    }

    /** Sets the currency money fields use when a value names none; empty removes it. */
    fun saveCurrency(typed: String) {
        val currency = typed.trim().takeIf { it.isNotEmpty() }?.let { CurrencyCodeModel.parse(raw = it) }
        if (typed.isNotBlank() && currency == null) {
            session.report(
                message = UiMessageModel(text = "Write a three-letter currency code such as EUR", isError = true),
            )
            return
        }
        change(subject = "Default currency", done = "Saved the default currency") { settings ->
            settings.withLayout(layout = settings.layout.copy(defaultCurrency = currency))
        }
    }

    private fun change(subject: String, done: String, change: (CatalogSettingsModel) -> CatalogSettingsModel) {
        session.perform(label = "Saving the library settings") {
            val outcome = services.documents.changeSettings(subject = subject, change = change)
            session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
        }
    }

    /** Whether approved metadata is also written into PDF files, as a new version of each. */
    fun setEmbedMetadata(enabled: Boolean) {
        change(
            subject = "Embedded PDF metadata",
            done = if (enabled) "PDFs get the approved metadata" else "PDFs stay as they are",
        ) {
            it.withLayout(layout = it.layout.copy(embedMetadata = enabled))
        }
    }

    fun categoryName(id: CategoryIdModel): String {
        return services.library.catalog.value?.categories?.get(id)?.name ?: id.value
    }

    fun setRuleEnabled(id: MatchingRuleIdModel, enabled: Boolean) {
        val rule = state.value.settings?.rules?.firstOrNull { it.id == id } ?: return
        change(
            subject = "Rule ${rule.name}",
            done = if (enabled) "Switched ${rule.name} on" else "Switched ${rule.name} off",
        ) {
            it.withRule(rule = rule.copy(enabled = enabled))
        }
    }

    fun removeRule(id: MatchingRuleIdModel) {
        change(subject = "Rule", done = "Removed the rule") { it.withoutRule(id = id) }
    }

    /** Why [path] cannot be the inbox, or `null` if it can. */
    fun inboxPathProblem(path: String): String? {
        val segments = path.split('/').map { it.trim() }.filter { it.isNotEmpty() }
        if (segments.isEmpty()) {
            return "Name a folder inside the library, such as Inbox"
        }
        val violation = segments.firstNotNullOfOrNull { FileNamePolicy.check(name = it) }
        return violation?.let { OutcomeMessages.nameProblem(violation = it) }
    }

    /** Saves the inbox settings; switching it on creates the folder in every library folder. */
    fun saveInbox(
        enabled: Boolean,
        path: String,
        aiAnalysis: Boolean,
        aiTargetFolder: Boolean,
        autoAnalyze: Boolean,
    ) {
        if (inboxPathProblem(path = path) != null) {
            return
        }
        val relative = RelativePathModel.fromSegments(
            segments = path.split('/').map { it.trim() }.filter { it.isNotEmpty() },
        )
        val inbox = InboxSettingsModel(
            enabled = enabled,
            path = relative,
            aiAnalysis = aiAnalysis,
            aiTargetFolder = aiTargetFolder,
            autoAnalyze = autoAnalyze,
        )
        session.perform(label = "Saving the inbox settings") {
            val outcome = services.organize.configureInbox(inbox = inbox)
            val done = if (enabled) "The inbox is ${inbox.path} in every library folder" else "The inbox is off"
            session.report(message = OutcomeMessages.of(outcome = outcome, done = done))
            session.rescan()
        }
    }
}
