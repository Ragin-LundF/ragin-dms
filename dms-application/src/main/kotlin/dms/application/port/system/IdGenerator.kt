package dms.application.port.system

/** Creates new unique ids for documents, categories, folders, libraries and journal entries. */
fun interface IdGenerator {

    fun newId(): String
}
