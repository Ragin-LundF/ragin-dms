import CATEGORY from "@material-design-icons/svg/round/category.svg?raw";
import LOCAL_OFFER from "@material-design-icons/svg/round/local_offer.svg?raw";
import BAR_CHART from "@material-design-icons/svg/round/bar_chart.svg?raw";
import PIE_CHART from "@material-design-icons/svg/round/pie_chart.svg?raw";
import BUSINESS from "@material-design-icons/svg/round/business.svg?raw";
import type {ChartPointDto, KindCountDto} from "../../../ipc";
import {AppCard, CardTitle, KIND_STYLES} from "../../../shared/components";
import {BarList} from "../charts/bar-list";
import {ColumnChart} from "../charts/column-chart";
import {DonutChart} from "../charts/donut-chart";

function Empty({ text }: { text: string }) {
  return <span className="body-medium muted">{text}</span>;
}

/** The largest categories and the uncategorized documents (id ""); a click opens them (Kotlin `CategoryCard`). */
export function CategoryCard({ points, onOpenCategory, onOpenUncategorized }: { points: ChartPointDto[]; onOpenCategory: (id: string) => void; onOpenUncategorized: () => void }) {
  return (
    <AppCard className="dashboard-chart-card">
      <CardTitle title="Categories" icon={CATEGORY} tint="var(--tertiary)" subtitle="Click one to open its documents" />
      {points.length === 0 && <Empty text="No documents yet" />}
      <BarList points={points} onClick={(point) => (point.id === "" ? onOpenUncategorized() : onOpenCategory(point.id))} />
    </AppCard>
  );
}

/** The tags as light chips, most used first, each with its count (Kotlin `TagCloudCard`). */
export function TagCloudCard({ points, onOpenTag }: { points: ChartPointDto[]; onOpenTag: (tag: string) => void }) {
  return (
    <AppCard className="dashboard-chart-card">
      <CardTitle title="Tags" icon={LOCAL_OFFER} tint="var(--secondary)" subtitle="Most used first" />
      {points.length === 0 && <Empty text="No tags yet" />}
      <div className="tag-cloud scroll">
        {points.map((point) => (
          <button key={point.id} type="button" className="tag-cloud-chip" onClick={() => onOpenTag(point.id)}>
            <span className="label-large nowrap">{point.label}</span>
            <span className="label-medium muted nowrap">{point.value}</span>
          </button>
        ))}
      </div>
    </AppCard>
  );
}

/** How many documents each year holds; a click on a year opens it (Kotlin `YearChartCard`). */
export function YearChartCard({ points, onOpenYear }: { points: ChartPointDto[]; onOpenYear: (id: string) => void }) {
  return (
    <AppCard className="dashboard-chart-card">
      <CardTitle title="Documents by year" icon={BAR_CHART} tint="var(--chart-1)" subtitle="By document date, or by when they were added. Click a year to open it" />
      {points.length === 0 ? <Empty text="No documents yet" /> : <ColumnChart points={points} color="var(--chart-1)" onClick={(point) => onOpenYear(point.id)} />}
    </AppCard>
  );
}

/** The share of PDFs, images, Office files and others (Kotlin `KindCard`). */
export function KindCard({ kinds }: { kinds: KindCountDto[] }) {
  const slices = kinds.map((entry) => ({ label: KIND_STYLES[entry.kind].label, value: entry.count, color: KIND_STYLES[entry.kind].color }));
  const total = kinds.reduce((sum, entry) => sum + entry.count, 0);
  return (
    <AppCard className="dashboard-chart-card">
      <CardTitle title="File types" icon={PIE_CHART} tint="var(--chart-2)" />
      <DonutChart slices={slices} centerValue={String(total)} centerLabel="files" />
    </AppCard>
  );
}

/** Who the documents come from; a click finds that correspondent's documents (Kotlin `CorrespondentCard`). */
export function CorrespondentCard({ points, onSearch }: { points: ChartPointDto[]; onSearch: (name: string) => void }) {
  return (
    <AppCard className="dashboard-chart-card">
      <CardTitle title="Correspondents" icon={BUSINESS} tint="var(--primary)" />
      {points.length === 0 && <Empty text="Add a correspondent in a document's details to see it here" />}
      <BarList points={points} onClick={(point) => onSearch(point.label)} />
    </AppCard>
  );
}
