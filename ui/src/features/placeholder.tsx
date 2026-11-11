import BUILD from "@material-design-icons/svg/round/construction.svg?raw";
import { EmptyState } from "../shared/components";

/** Stands in for a screen that is not ported yet. */
export function Placeholder({ title }: { title: string }) {
  return <EmptyState icon={BUILD} title={title} message="This screen is not ported yet." style={{ height: "100%" }} />;
}
