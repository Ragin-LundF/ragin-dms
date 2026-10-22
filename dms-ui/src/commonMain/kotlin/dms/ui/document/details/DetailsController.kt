package dms.ui.document.details

import dms.application.common.OperationOutcome
import dms.application.document.model.DetailEdit
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldType
import dms.domain.field.FieldValue
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldKeyModel
import dms.domain.retention.model.RetentionModel
import dms.domain.rule.RuleCondition
import dms.domain.rule.RuleOrigin
import dms.domain.rule.model.CorrespondentModel
import dms.domain.rule.model.MatchingRuleIdModel
import dms.domain.rule.model.MatchingRuleModel
import dms.domain.rule.model.RuleActionsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.ui.document.details.model.LinkCandidateModel
import dms.ui.document.details.model.LinkPickModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.time.LocalDate
import java.util.Locale
import java.util.UUID

/**
 * The details of the document in the inspector beyond its metadata: custom fields, the note, the
 * linked documents and the opt-in retention. The inspector's view model owns it; every change is one
 * catalog change through the session.
 */
class DetailsController(
    private val services: AppServices,
    private val session: LibrarySession,
) {

    private val pickState = MutableStateFlow<LinkPickModel?>(value = null)

    /** The open "Link to…" picker, if any. */
    val linkPick: StateFlow<LinkPickModel?> = pickState.asStateFlow()

    /** Creates the standard fields (amount, due date, cancel by, contract end, warranty) on first use. */
    fun addStandardFields() {
        session.perform(label = "Adding the standard fields") {
            val outcome = services.documents.changeSettings(subject = "Standard fields") { settings ->
                FieldPresets.ALL.fold(initial = settings) { current, preset ->
                    if (current.field(key = preset.key) == null) current.withField(definition = preset) else current
                }
            }
            session.report(message = OutcomeMessages.of(outcome = outcome, done = "Added the standard fields"))
        }
    }

    /**
     * Saves the typed field values: empty ones are removed, the others must fit their field's type,
     * or nothing is saved and the message names the field.
     */
    fun saveFields(document: DocumentModel, typed: Map<FieldKeyModel, String>) {
        val catalog = services.library.catalog.value ?: return
        val currency = catalog.settings.layout.defaultCurrency
        val values = mutableMapOf<FieldKeyModel, FieldValue>()
        typed.forEach { (key, raw) ->
            val definition = catalog.settings.field(key = key) ?: return@forEach
            if (raw.isBlank()) {
                return@forEach
            }
            val value = FieldValues.parse(type = definition.type, raw = raw, defaultCurrency = currency)
            if (value == null) {
                session.report(
                    message = UiMessageModel(
                        text = "\"${definition.label}\": ${hintOf(type = definition.type)}",
                        isError = true,
                    ),
                )
                return
            }
            values[key] = value
        }
        val kept = document.fields.filterKeys { catalog.settings.field(key = it) == null }
        edit(edit = DetailEdit.SetFields(documentId = document.id, fields = kept + values))
    }

    fun saveNote(document: DocumentModel, note: String) {
        if (note.trimEnd() == document.note.orEmpty()) {
            return
        }
        edit(edit = DetailEdit.SetNote(documentId = document.id, note = note))
    }

    /** Sets the keep-until date (`YYYY-MM-DD`), or removes it when empty. A done retention stays done. */
    fun saveKeepUntil(document: DocumentModel, typed: String) {
        val date = typed.trim().takeIf { it.isNotEmpty() }?.let { runCatching { LocalDate.parse(it) }.getOrNull() }
        if (typed.isNotBlank() && date == null) {
            session.report(message = UiMessageModel(text = "The date must look like 2030-12-31", isError = true))
            return
        }
        val retention = (document.retention ?: RetentionModel()).copy(keepUntil = date)
        edit(edit = DetailEdit.SetRetention(documentId = document.id, retention = retention))
    }

    /** Sets how many years documents of [categoryId] are kept; empty removes the period. */
    fun saveCategoryYears(categoryId: CategoryIdModel, typed: String) {
        val years = typed.trim().takeIf { it.isNotEmpty() }?.toIntOrNull()
        if (typed.isNotBlank() && (years == null || years !in 1..CategoryModel.MAX_RETENTION_YEARS)) {
            session.report(message = UiMessageModel(text = "Write a number of years from 1 to 100", isError = true))
            return
        }
        edit(edit = DetailEdit.SetRetentionYears(categoryId = categoryId, years = years))
    }

    /**
     * A manual rule from this document: documents of its correspondent get its category and tags
     * proposed. The correspondent becomes a known one, so it is recognized in other documents' text.
     */
    fun createRule(document: DocumentModel) {
        val correspondent = document.metadata.correspondent
        if (correspondent == null || (document.categoryId == null && document.tags.isEmpty())) {
            session.report(
                message = UiMessageModel(
                    text = "Give the document a correspondent and a category or tags first",
                    isError = true,
                ),
            )
            return
        }
        val rule = MatchingRuleModel(
            id = MatchingRuleIdModel(value = UUID.randomUUID().toString()),
            name = "Rule for $correspondent",
            conditions = listOf(RuleCondition.CorrespondentIs(name = correspondent)),
            actions = RuleActionsModel(categoryId = document.categoryId, tags = document.tags),
            origin = RuleOrigin.MANUAL,
        )
        session.perform(label = "Creating ${rule.name}") {
            val outcome = services.documents.changeSettings(subject = rule.name) { settings ->
                val known = settings.correspondents.any { it.key == CorrespondentModel(name = correspondent).key }
                val withName = if (known) settings else settings.withCorrespondent(
                    correspondent = CorrespondentModel(name = correspondent),
                )
                withName.withRule(rule = rule)
            }
            session.report(message = OutcomeMessages.of(outcome = outcome, done = "Created ${rule.name}"))
        }
    }

    fun unlink(document: DocumentModel, other: DocumentIdModel) {
        edit(edit = DetailEdit.Unlink(first = document.id, second = other))
    }

    fun startLink(document: DocumentModel) {
        pickState.value = LinkPickModel(documentId = document.id)
        search(query = "")
    }

    /** Finds documents to link by title, name, path or correspondent; the document itself is left out. */
    fun search(query: String) {
        val pick = pickState.value ?: return
        val catalog = services.library.catalog.value ?: return
        val wanted = query.trim().lowercase(Locale.ROOT)
        val matches = catalog.documents.values
            .filter { it.id != pick.documentId }
            .filter { wanted.isEmpty() || textOf(document = it).contains(other = wanted) }
            .sortedByDescending { it.addedAt }
            .take(n = MATCHES)
            .map { candidateOf(document = it) }
        pickState.value = pick.copy(query = query, matches = matches)
    }

    fun setLinkLabel(label: String) {
        pickState.value = pickState.value?.copy(label = label)
    }

    fun link(other: DocumentIdModel) {
        val pick = pickState.value ?: return
        pickState.value = null
        edit(
            edit = DetailEdit.Link(first = pick.documentId, second = other, label = pick.label.trim().ifEmpty { null }),
        )
    }

    fun cancelLink() {
        pickState.value = null
    }

    /** The documents closest in meaning, from the index; empty while the index has no entry for it. */
    suspend fun related(document: DocumentModel): List<LinkCandidateModel> {
        val catalog = services.library.catalog.value ?: return emptyList()
        return services.ask.relatedDocuments(documentId = document.id).mapNotNull { id ->
            catalog.documents[id]?.let { candidateOf(document = it) }
        }
    }

    /** The documents linked with [document], with their labels. */
    fun linksOf(document: DocumentModel, catalog: CatalogModel): List<LinkCandidateModel> {
        return catalog.linksOf(documentId = document.id).mapNotNull { link ->
            catalog.documents[link.otherThan(documentId = document.id)]?.let { candidateOf(
                document = it,
            ).copy(label = link.label) }
        }.sortedBy { it.name.lowercase(Locale.ROOT) }
    }

    private fun candidateOf(document: DocumentModel): LinkCandidateModel {
        return LinkCandidateModel(
            documentId = document.id,
            name = document.metadata.title ?: document.path.fileName,
            path = document.path.value,
        )
    }

    private fun textOf(document: DocumentModel): String {
        return listOfNotNull(document.path.value, document.metadata.title, document.metadata.correspondent)
            .joinToString(separator = " ")
            .lowercase(Locale.ROOT)
    }

    private fun hintOf(type: FieldType): String {
        return when (type) {
            FieldType.TEXT -> "write some text"
            FieldType.NUMBER -> "write a number"
            FieldType.MONEY -> "write an amount, such as 12.50 EUR"
            FieldType.DATE -> "write a date such as 2026-12-31"
            FieldType.BOOLEAN -> "write yes or no"
        }
    }

    private fun edit(edit: DetailEdit) {
        session.perform(label = "Saving the catalog") {
            val outcome = services.documents.editDocumentDetails(edit = edit)
            if (outcome !is OperationOutcome.Done) {
                session.report(message = OutcomeMessages.of(outcome = outcome, done = ""))
            }
        }
    }

    private companion object {
        const val MATCHES = 30
    }
}
