package dms.storage.catalog.dto

/**
 * A part of the catalog file that keeps the fields only a newer version knows, so saving with this
 * version does not drop them. Every implementation annotates [unknownFields] with `@JsonAnyGetter` and
 * `@JsonAnySetter`.
 */
interface ExtensibleDto {
    val unknownFields: MutableMap<String, Any?>
}
