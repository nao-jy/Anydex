import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Accordion,
  Box,
  Button,
  Card,
  Container,
  Divider,
  Group,
  Paper,
  SimpleGrid,
  Stack,
  Text,
  ThemeIcon,
  Title,
} from "@mantine/core";
import { commands } from "../bindings";

type SortKey = "name" | "createdAt" | "updatedAt";
type Direction = "asc" | "desc";

export const Index = () => {
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [direction, setDirection] = useState<Direction>("desc");
  const [open, setOpen] = useState<string[]>([]);
  const {
    data: indexes = [],
    isLoading,
    isError,
  } = useQuery({ queryKey: ["indexes"], queryFn: commands.listIndexes });
  const sortedIndexes = useMemo(
    () =>
      [...indexes].sort((a, b) => {
        const result = a[sortKey].localeCompare(b[sortKey], "ja");
        return direction === "asc" ? result : -result;
      }),
    [sortKey, direction, indexes],
  );
  const chooseSort = (key: SortKey) => {
    if (sortKey === key)
      setDirection((value) => (value === "asc" ? "desc" : "asc"));
    else {
      setSortKey(key);
      setDirection(key === "name" ? "asc" : "desc");
    }
  };

  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <Group justify="space-between" align="flex-end" mb={28}>
          <Box>
            <Text size="sm" c="indigo" fw={700} tt="uppercase" lts={1}>
              Search indexes
            </Text>
            <Title order={1} mt={4}>
              インデックス
            </Title>
            <Text c="dimmed" mt={6}>
              ファイルソースと検索設定を管理します
            </Text>
          </Box>
          <Button size="md">＋ 新しいインデックス</Button>
        </Group>
        <Card withBorder radius="md" padding="sm" mb="md">
          <Group justify="space-between">
            <Group gap={4}>
              <Text size="sm" fw={600} c="dimmed" mr="xs">
                並び替え
              </Text>
              {(
                [
                  ["name", "名前"],
                  ["createdAt", "作成日"],
                  ["updatedAt", "最終更新日"],
                ] as [SortKey, string][]
              ).map(([key, label]) => (
                <Button
                  key={key}
                  variant={sortKey === key ? "light" : "subtle"}
                  color="gray"
                  size="compact-sm"
                  onClick={() => chooseSort(key)}
                >
                  {label}
                  {sortKey === key ? (direction === "asc" ? " ↑" : " ↓") : ""}
                </Button>
              ))}
            </Group>
            <Text size="sm" c="dimmed">
              {indexes.length} 件のインデックス
            </Text>
          </Group>
        </Card>
        {isLoading && (
          <Text c="dimmed" ta="center" py="xl">
            インデックスを読み込んでいます…
          </Text>
        )}
        {isError && (
          <Text c="red" ta="center" py="xl">
            インデックスを取得できませんでした。
          </Text>
        )}
        {!isLoading && !isError && (
          <Accordion
            multiple
            value={open}
            onChange={setOpen}
            variant="separated"
            radius="md"
          >
            {sortedIndexes.map((index) => (
              <Accordion.Item value={index.id.toString()} key={index.id}>
                <Accordion.Control>
                  <Group justify="space-between" wrap="nowrap" pr="md">
                    <Box>
                      <Group gap="sm">
                        <Text fw={700} size="lg">
                          {index.name}
                        </Text>
                      </Group>
                      <Text size="sm" c="dimmed" mt={3}>
                        {index.description}
                      </Text>
                    </Box>
                    <Group gap="xl" visibleFrom="sm">
                      <DateInfo label="作成日" value={index.createdAt} />
                      <DateInfo label="最終更新日" value={index.updatedAt} />
                    </Group>
                  </Group>
                </Accordion.Control>
                <Accordion.Panel>
                  <Stack gap="lg" pb="sm">
                    <SimpleGrid cols={{ base: 1, sm: 3 }}>
                      <Metric
                        label="Indexer"
                        value={`${index.indexers.length} 件`}
                      />
                      <Metric
                        label="前回の処理ファイル数"
                        value={`${index.indexers.reduce((total, item) => total + (item.lastProcessedFiles ?? 0), 0)} 件`}
                      />
                      <Metric label="ベクトル化" value={index.embedder.name} />
                    </SimpleGrid>
                    <Divider />
                    <Group justify="space-between">
                      <Text fw={700}>Indexer</Text>
                      <Button variant="subtle" size="compact-sm">
                        ＋ 追加
                      </Button>
                    </Group>
                    <Stack gap="sm">
                      {index.indexers.map((indexer, position) => (
                        <Paper
                          withBorder
                          p="md"
                          radius="md"
                          key={`${indexer.fileSource.name}-${position}`}
                        >
                          <Group justify="space-between" align="flex-start">
                            <Group align="flex-start" gap="sm">
                              <ThemeIcon variant="light" color="indigo">
                                ↗
                              </ThemeIcon>
                              <Box>
                                <Text fw={600}>{indexer.fileSource.name}</Text>
                                <Text size="xs" c="dimmed" mt={4}>
                                  同期:{" "}
                                  {formatSchedule(indexer.schedule.rfc5545Rule)}
                                  （
                                  {formatDateTime(
                                    indexer.schedule.dtstart,
                                    indexer.schedule.timezone,
                                  )}{" "}
                                  起点）
                                </Text>
                              </Box>
                            </Group>
                            <Box ta="right">
                              <Text size="sm">
                                {indexer.lastProcessedFiles == null
                                  ? "未実行"
                                  : `${indexer.lastProcessedFiles} ファイル`}
                              </Text>
                              <Text size="xs" c="dimmed">
                                最終成功:{" "}
                                {indexer.lastSuccess
                                  ? formatDateTime(
                                      indexer.lastSuccess,
                                      indexer.schedule.timezone,
                                    )
                                  : "なし"}
                              </Text>
                            </Box>
                          </Group>
                        </Paper>
                      ))}
                    </Stack>
                    <Setting
                      label="ベクトル化"
                      value={index.embedder.name}
                      detail="OpenAPI エンドポイント"
                    />
                    <Group justify="flex-end">
                      <Button variant="default">設定を編集</Button>
                      <Button>今すぐ同期</Button>
                    </Group>
                  </Stack>
                </Accordion.Panel>
              </Accordion.Item>
            ))}
          </Accordion>
        )}
      </Container>
    </Box>
  );
};

function DateInfo({ label, value }: { label: string; value: string }) {
  return (
    <Box>
      <Text size="xs" c="dimmed">
        {label}
      </Text>
      <Text size="sm">{value}</Text>
    </Box>
  );
}
function Metric({ label, value }: { label: string; value: string }) {
  return (
    <Paper p="md" radius="md">
      <Text size="xs" c="dimmed">
        {label}
      </Text>
      <Text fw={700} size="lg" mt={3}>
        {value}
      </Text>
    </Paper>
  );
}
function Setting({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <Paper withBorder p="md" radius="md">
      <Text size="xs" c="dimmed">
        {label}
      </Text>
      <Text fw={600} mt={3}>
        {value}
      </Text>
      <Text size="xs" c="dimmed" mt={4}>
        {detail}
      </Text>
    </Paper>
  );
}

const sourceTypeLabels: Record<string, string> = {
  fs: "Filesystem",
  s3: "Amazon S3",
  gcs: "Google Cloud Storage",
  azblob: "Azure Blob Storage",
  webdav: "WebDAV",
  http: "HTTP",
  https: "HTTPS",
};
const weekdays: Record<string, string> = {
  MO: "月",
  TU: "火",
  WE: "水",
  TH: "木",
  FR: "金",
  SA: "土",
  SU: "日",
};

function formatSourceType(uri: string) {
  const scheme = uri.match(/^([a-z][a-z0-9+.-]*):/i)?.[1]?.toLowerCase();
  return scheme ? (sourceTypeLabels[scheme] ?? scheme) : "Unknown";
}

function formatSchedule(rule: string) {
  const fields = Object.fromEntries(
    rule
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
  return rule;
}

function formatDateTime(value: string, timezone: string) {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) return value;
  return new Intl.DateTimeFormat("ja-JP", {
    dateStyle: "short",
    timeStyle: "short",
    timeZone: timezone,
  }).format(date);
}
