package dms.application.library.model

import dms.domain.library.LibraryFolders

sealed interface ConfigureLibraryOutcome {

    data class Configured(val folders: LibraryFolders) : ConfigureLibraryOutcome

    data object AlreadyConfigured : ConfigureLibraryOutcome

    data object NotConfigured : ConfigureLibraryOutcome

    data object LocationAlreadyUsed : ConfigureLibraryOutcome

    data object UnknownFolder : ConfigureLibraryOutcome

    data object PrimaryCannotBeRemoved : ConfigureLibraryOutcome
}
