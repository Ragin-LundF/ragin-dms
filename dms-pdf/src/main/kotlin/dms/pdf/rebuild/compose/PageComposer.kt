package dms.pdf.rebuild.compose

import dms.domain.rebuild.KeepReason
import dms.domain.rebuild.RegionKind
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.PageLayoutModel
import dms.domain.rebuild.model.RebuildRegionModel
import dms.domain.rebuild.model.RebuiltPageModel
import dms.pdf.rebuild.PixelBoxes
import dms.pdf.rebuild.compose.model.CropModel
import dms.pdf.rebuild.fit.LineFitter
import dms.pdf.rebuild.fit.model.FittedLine
import dms.pdf.rebuild.font.FontSet
import dms.pdf.rebuild.image.PageMasks
import org.apache.pdfbox.pdmodel.PDDocument
import java.awt.image.BufferedImage

/** Rebuilds one straightened page from its layout: masks, fitted lines, crops, and the written page. */
object PageComposer {

    fun add(
        document: PDDocument,
        fonts: FontSet,
        page: BufferedImage,
        angle: Double,
        layout: PageLayoutModel,
    ): RebuiltPageModel {
        val masks = PageMasks(page = page)
        val boxes = PixelBoxes(width = page.width, height = page.height)
        val lines = LineFitter(masks = masks, fonts = fonts).fit(layout = layout, boxes = boxes)
        val crops = Crops.of(masks = masks, lines = lines)
        PageWriter.add(
            document = document,
            fonts = fonts,
            width = page.width,
            height = page.height,
            crops = crops,
            lines = lines,
        )
        return summaryOf(lines = lines, crops = crops, angle = angle, boxes = boxes)
    }

    /** A page the AI could not read, kept as it was. */
    fun failed(): RebuiltPageModel {
        return RebuiltPageModel(
            linesAsText = 0,
            linesTotal = 0,
            angle = 0.0,
            keptReasons = mapOf(KeepReason.PAGE_FAILED to 1),
            regions = listOf(
                RebuildRegionModel(
                    box = LayoutBoxModel(
                        left = 0.0,
                        top = 0.0,
                        right = LayoutBoxModel.SCALE,
                        bottom = LayoutBoxModel.SCALE,
                    ),
                    kind = RegionKind.IMAGE,
                ),
            ),
        )
    }

    private fun summaryOf(
        lines: List<FittedLine>,
        crops: List<CropModel>,
        angle: Double,
        boxes: PixelBoxes,
    ): RebuiltPageModel {
        val kept = lines.filterIsInstance<FittedLine.Kept>()
        val regions = lines.map { line ->
            val kind = if (line is FittedLine.Text) RegionKind.TEXT else RegionKind.KEPT_LINE
            RebuildRegionModel(box = boxes.toLayout(box = line.box), kind = kind)
        } + crops.map { RebuildRegionModel(box = boxes.toLayout(box = it.box), kind = RegionKind.IMAGE) }
        return RebuiltPageModel(
            linesAsText = lines.size - kept.size,
            linesTotal = lines.size,
            angle = angle,
            keptReasons = kept.groupingBy { it.reason }.eachCount(),
            regions = regions,
        )
    }
}
