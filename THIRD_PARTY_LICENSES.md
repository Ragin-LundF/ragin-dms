# Third-party licenses

## PDFium (PDF rendering, text and editing)

ragin-dms loads the PDFium library at runtime (`crates/infra-pdf`). The prebuilt binaries come from
[bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries), release `chromium/8125`, fetched by
`scripts/fetch-pdfium.sh` into `vendor/pdfium/<platform>/` and shipped with the app.

- PDFium itself: BSD 3-Clause license, © The PDFium Authors (parts under Apache 2.0).
- The binary build scripts: MIT license, © Benoit Blanchon.
- PDFium bundles third-party code with its own permissive licenses (FreeType, HarfBuzz, ICU, Little CMS, libjpeg-turbo,
  libpng, OpenJPEG, zlib, Abseil, AGG, fast_float, Highway, simdutf, dragonbox, llvm-libc).

The full license texts are in each fetched folder: `vendor/pdfium/<platform>/LICENSE` and
`vendor/pdfium/<platform>/licenses/`. A distributed app must ship these files next to the library.

## Liberation fonts (rebuilt PDFs and invisible text layers)

`crates/infra-pdf/fonts/` holds Liberation Sans, Serif and Mono (regular and bold), © 2010 Google Corporation,
© 2012 Red Hat, Inc., under the SIL Open Font License 1.1. They are embedded into the PDFs the app writes. The license
text is `crates/infra-pdf/fonts/LICENSE-Liberation.txt`.

## Geist font (user interface)

`ui/src/assets/fonts/` holds Geist (regular, medium, semibold, bold), © 2023 Vercel, under the SIL Open Font License
1.1, as in the Kotlin app. The license text is `ui/src/assets/fonts/geist-OFL.txt`; it is bundled with the UI.

## Material Design icons (user interface)

The icons come from `@material-design-icons/svg` (round set), © Google, Apache License 2.0.

## Rust crates

Linked into the app; each crate's license text is in its source package (`cargo metadata`, crates.io).

| Crate | Used by | License |
|---|---|---|
| tauri, tauri-build, tauri-plugin-dialog, tauri-plugin-opener | desktop shell | Apache-2.0 OR MIT |
| tauri-specta, specta, specta-typescript | typed commands | MIT |
| percent-encoding, iana-time-zone | desktop shell | MIT OR Apache-2.0 |
| fast_qr | payment QR codes (GiroCode) as SVG | MIT |
| rfd | crash dialog | MIT |
| ureq, base64 | AI adapters (HTTP, images) | MIT OR Apache-2.0 |
| calamine | XLSX text for search and AI | MIT |
| zip | Office files, exports, the encrypted emergency ZIP | MIT |
| aes, pbkdf2, hmac, sha1 | AES-256 encryption of the emergency ZIP (through zip) | MIT OR Apache-2.0 |
| mail-parser | reading emails (.eml) for the email import | Apache-2.0 OR MIT |
| quick-xml | DOCX/PPTX text | MIT |
| saphyr-parser | catalog and settings YAML | MIT OR Apache-2.0 |
| notify | folder watcher | CC0-1.0 |
| trash | system trash | MIT |
| rustix | file system (macOS, Linux) | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| sha2 | content hashes | MIT OR Apache-2.0 |
| pdfium-render | PDF engine binding | MIT OR Apache-2.0 |
| lopdf | PDF writing | MIT |
| image, ttf-parser | page pictures, fonts | MIT OR Apache-2.0 |
| chrono, chrono-tz, indexmap, uuid, serde, serde_json, thiserror, num-bigint | everywhere | MIT OR Apache-2.0 |
| unicode-normalization | file names | (MIT OR Apache-2.0) AND Unicode-3.0 |

Their transitive dependencies (the Tauri/wry/tao stack among them) are not listed one by one here; check them with
`cargo license` (or `cargo about`) before a public release.

## JavaScript packages (user interface)

Bundled into `ui/dist`.

| Package | License |
|---|---|
| react, react-dom | MIT |
| @tanstack/react-query, @tanstack/react-virtual | MIT |
| @tauri-apps/api, @tauri-apps/plugin-dialog | Apache-2.0 OR MIT |
| @material-design-icons/svg | Apache-2.0 |
