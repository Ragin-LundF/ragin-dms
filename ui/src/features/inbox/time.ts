/** When a document arrived, as the Kotlin activity texts write it: `dd.MM. HH:mm`, local time. */
export function arrivalTime(instant: string): string {
  const date = new Date(instant);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${pad(date.getDate())}.${pad(date.getMonth() + 1)}. ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}
