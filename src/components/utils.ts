const weekdays: Record<string, string> = {
  MO: "月",
  TU: "火",
  WE: "水",
  TH: "木",
  FR: "金",
  SA: "土",
  SU: "日",
};

export function formatSchedule(rrule: string) {
  const fields = Object.fromEntries(
    rrule
      .split(";")
      .map((field) => field.split("=", 2))
      .filter((field): field is [string, string] => field.length === 2),
  );
  const interval =
    fields.INTERVAL && fields.INTERVAL !== "1" ? Number(fields.INTERVAL) : 1;
  if (fields.FREQ === "DAILY")
    return interval === 1 ? "毎日" : `${interval} 日おき`;
  if (fields.FREQ === "WEEKLY") {
    const days = fields.BYDAY?.split(",")
      .map((day) => weekdays[day] ?? day)
      .join("・");
    return `${interval === 1 ? "毎週" : `${interval} 週おき`}${days ? ` ${days}曜` : ""}`;
  }
  if (fields.FREQ === "MONTHLY")
    return interval === 1 ? "毎月" : `${interval} か月おき`;
  return rrule;
}

export function formatDateTime(value: string, timezone?: string) {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) return value;
  return new Intl.DateTimeFormat("ja-JP", {
    dateStyle: "short",
    timeStyle: "short",
    timeZone: timezone,
  }).format(date);
}
