package dms.storage.yaml

import tools.jackson.databind.DeserializationFeature
import tools.jackson.dataformat.yaml.YAMLMapper
import tools.jackson.dataformat.yaml.YAMLWriteFeature
import tools.jackson.module.kotlin.KotlinModule

/** The one YAML configuration for every file the tool writes: plain, readable, diff-friendly. */
object YamlMappers {

    val yaml: YAMLMapper = YAMLMapper.builder()
        .addModule(KotlinModule.Builder().build())
        .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
        .disable(YAMLWriteFeature.WRITE_DOC_START_MARKER)
        .enable(YAMLWriteFeature.MINIMIZE_QUOTES)
        // Multi-line text such as a document's note is written as a readable block scalar.
        .enable(YAMLWriteFeature.LITERAL_BLOCK_STYLE)
        .enable(YAMLWriteFeature.ALWAYS_QUOTE_NUMBERS_AS_STRINGS)
        .build()
}
