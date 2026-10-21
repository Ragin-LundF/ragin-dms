# Static Analysis Instructions

Use this file when fixing Detekt findings, changing style-sensitive code, or adding suppressions.

## Detekt

- Detekt is configured in `config/detekt.yml` and applied to every module through `config/gradle/static_analysis.gradle`.
- The Detekt source set lists both the plain JVM layout (`src/main/kotlin`, `src/test/kotlin`) and the Kotlin Multiplatform layout (`src/commonMain/kotlin`, `src/jvmMain/kotlin`, `src/jvmTest/kotlin`). A new source set must be added there, or it is silently not analysed.
- All code must pass Detekt checks.
- Prefer fixing the underlying issue instead of suppressing it.
- Use `@Suppress` only for individual, justified suppressions.
- Suppressions must name the specific rule, for example `@Suppress("TooManyFunctions")`.
- Do not suppress whole files.
- Add a short reason near the suppression when the justification is not obvious.
- Composables are PascalCase by framework convention; `FunctionNaming` and `MagicNumber` are configured for `@Composable` in `config/detekt.yml`. Do not suppress them per function.
