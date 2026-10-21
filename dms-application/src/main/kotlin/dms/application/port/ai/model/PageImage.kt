package dms.application.port.ai.model

/** One rendered page or image, already downscaled for the model. */
class PageImage(
    val bytes: ByteArray,
    val mimeType: String,
)
