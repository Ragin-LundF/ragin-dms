import PDF from "@material-design-icons/svg/round/picture_as_pdf.svg?raw";
import IMAGE from "@material-design-icons/svg/round/image.svg?raw";
import OFFICE from "@material-design-icons/svg/round/description.svg?raw";
import OTHER from "@material-design-icons/svg/round/insert_drive_file.svg?raw";
import {IconBadge} from "./cards";

/** The kinds of documents (domain `DocumentKind`) as the DTOs name them. */
export type DocumentKind = "pdf" | "image" | "office" | "other";

/** Name, icon and colour of each kind of document; the colour is the same in every chart and badge. */
export const KIND_STYLES: Record<DocumentKind, { label: string; icon: string; color: string }> = {
  pdf: { label: "PDF", icon: PDF, color: "var(--chart-2)" },
  image: { label: "Image", icon: IMAGE, color: "var(--chart-3)" },
  office: { label: "Office", icon: OFFICE, color: "var(--chart-1)" },
  other: { label: "Other", icon: OTHER, color: "var(--chart-neutral)" },
};

/** The coloured tile that tells PDFs, images and Office files apart at a glance. */
export function KindBadge({ kind, size = 36 }: { kind: DocumentKind; size?: number }) {
  const style = KIND_STYLES[kind];
  return <IconBadge icon={style.icon} tint={style.color} size={size} />;
}
