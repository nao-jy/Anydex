import { Temporal } from "temporal-polyfill";
import { Schedule } from "../bindings";
import { RRuleTemporal, TemporalZonedDateTime } from "rrule-temporal";
import { useEffect, useState } from "react";
import {
  Alert,
  Button,
  Checkbox,
  Code,
  Divider,
  Group,
  Input,
  NumberInput,
  Paper,
  Select,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { DateTimePicker } from "@mantine/dates";
import { CaretDownIcon, WarningIcon } from "@phosphor-icons/react";

const getDefaultStartDt = () =>
  Temporal.Now.zonedDateTimeISO().add({ days: 1 }).startOfDay();

const toDtStart = (zdt: TemporalZonedDateTime) => {
  const local = zdt.toPlainDateTime();

  const value = local.toString({ smallestUnit: "second" }).replace(/[-:]/g, "");

  return `DTSTART;TZID=${zdt.timeZoneId}:${value}`;
};

interface ScheduleProps {
  index_id: string;
  schedule: Schedule | null;
  onUpsertSchedule: (index_id: string, info: Schedule) => void;
  onDeleteSchedule: (index_id: string) => void;
}

export const ScheduleInput = (props: ScheduleProps) => {
  const { schedule, index_id, onUpsertSchedule, onDeleteSchedule } = props;
  const defaultRuleStr =
    schedule?.rfc5545Rrule ??
    `DTSTART;TZID=${toDtStart(getDefaultStartDt())}\nRRULE:FREQ=DAILY`;
  const rule = new RRuleTemporal({
    rruleString: defaultRuleStr,
  });
  const [rruleText, setRruleText] = useState(rule.toString());
  const [rrule, setRrule] = useState(rule);
  const rruleOptions = rrule.options();

  const frequency = [
    {
      label: "時間おき",
      value: "HOURLY",
    },
    {
      label: "日おき",
      value: "DAILY",
    },
    {
      label: "週おき",
      value: "WEEKLY",
    },
    {
      label: "月おき",
      value: "MONTHLY",
    },
    {
      label: "年おき",
      value: "YEARLY",
    },
  ];

  const [catchupWhenMissed, setCatchupWhenMissed] = useState(
    schedule?.catchupWhenMissed ?? false,
  );

  useEffect(() => {
    setRruleText(rrule.toString());
  }, [rrule.toString()]);

  return (
    <Paper withBorder p="md" radius="md">
      <Stack gap="md">
        <Text size="xs" c="dimmed">
          {"定期実行スケジュール"}
        </Text>
        <Group align="flex-start" wrap="nowrap">
          <DateTimePicker
            label={"開始日"}
            valueFormat="YYYY-MM-DD HH:mm"
            style={{ flex: 1 }}
            value={new Date(rruleOptions.dtstart.epochMilliseconds)}
          />
          <DateTimePicker
            label={"終了日"}
            valueFormat="YYYY-MM-DD HH:mm"
            style={{ flex: 1 }}
            value={
              rruleOptions.until === undefined
                ? null
                : new Date(rruleOptions.until.epochMilliseconds)
            }
            onChange={(v) => {
              const newDt =
                v === null
                  ? undefined
                  : Temporal.PlainDateTime.from(
                      v.replace(" ", "T"),
                    ).toZonedDateTime(Temporal.Now.timeZoneId());
              const newRule = new RRuleTemporal({
                ...rruleOptions,
                until: newDt,
              });
              setRrule(newRule);
            }}
            clearable
            rightSection={<CaretDownIcon size={16} />}
            clearSectionMode="clear"
          />
        </Group>
        <Group align="flex-start" wrap="nowrap">
          <Input.Wrapper label="頻度" style={{ flex: 1 }}>
            <Group align="flex-start" gap="md" wrap="nowrap">
              <NumberInput
                style={{ width: "100px" }}
                min={1}
                allowDecimal={false}
                value={rruleOptions.interval ?? 1}
                onChange={(v) => {
                  const newRule = new RRuleTemporal({
                    ...rruleOptions,
                    interval: Number(v),
                  });
                  setRrule(newRule);
                }}
              />
              <Select
                data={frequency}
                style={{ width: "200px" }}
                value={rruleOptions.freq}
                onChange={(v) => {
                  if (v === null) return;
                  const newRule = new RRuleTemporal({
                    ...rruleOptions,
                    freq: v as any,
                  });
                  setRrule(newRule);
                }}
              />
            </Group>
          </Input.Wrapper>
        </Group>
        <Checkbox
          label="非起動時に予定を過ぎた場合、次回起動時に処理を開始する"
          checked={catchupWhenMissed}
          onChange={(e) => {
            setCatchupWhenMissed(e.currentTarget.checked);
          }}
        />
        <Divider />
        <Group align="flex-start" wrap="nowrap">
          <TextInput
            label={"RFC5545 RRULE 直接入力"}
            value={rruleText}
            onChange={(e) => {
              const v = e.currentTarget.value;
              setRruleText(v);
              try {
                const newRule = new RRuleTemporal({
                  rruleString: v,
                });
                setRrule(newRule);
              } catch {}
            }}
            style={{ flex: 1 }}
          />
          <Group gap="xs" mt={24} wrap="nowrap">
            <Button
              variant="default"
              onClick={() => {
                const newRule = new RRuleTemporal({
                  rruleString: defaultRuleStr,
                });
                setRrule(newRule);
              }}
            >
              リセット
            </Button>
          </Group>
        </Group>
        <Input.Wrapper label={"実行予定日時プレビュー"}>
          <Code style={{ display: "block", whiteSpace: "pre-wrap" }}>
            {rrule
              .all((_, i) => i < 6)
              .map((v) => v.toLocaleString())
              .join("\n")}
          </Code>
        </Input.Wrapper>
        <Group gap="xs" mt={24} wrap="nowrap">
          <Button
            size="sm"
            onClick={async () => {
              const newSchedule: Schedule = {
                rfc5545Rrule: rrule.toString(),
                catchupWhenMissed: catchupWhenMissed,
              };
              onUpsertSchedule(index_id, newSchedule);
            }}
          >
            {"保存"}
          </Button>
          {schedule !== null && (
            <Button
              variant="outline"
              size="sm"
              onClick={async () => {
                onDeleteSchedule(index_id);
              }}
            >
              {"削除"}
            </Button>
          )}
          {schedule === null && (
            <Alert
              color="yellow"
              title="定期実行は未設定です"
              icon={<WarningIcon />}
            />
          )}
        </Group>
      </Stack>
    </Paper>
  );
};
