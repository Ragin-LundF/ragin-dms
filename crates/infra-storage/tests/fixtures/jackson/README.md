# Jackson golden files

Written by the released Kotlin 1.3.0 jars (`dms-storage`, Jackson 3.2.4, snakeyaml-engine 3.0.1) with
`generator/Driver.kt`, from synthetic inputs only (`../catalog/*.yaml` and the values in the driver). The Rust
tests compare their output with these bytes.

Regenerate (needs the 1.3.0 jars of ragin-dms and the Gradle cache): compile `Driver.kt` with
`kotlin-compiler-embeddable` 2.4.21 against those jars and run `DriverKt <catalog fixtures dir> <out dir>`.
`saved-*.yaml` are the catalog fixtures after one load and save through `FileCatalogRepository`.
