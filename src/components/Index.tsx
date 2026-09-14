import { useMemo, useState } from "react";
import { Accordion, Badge, Box, Button, Card, Container, Divider, Group, Paper, SimpleGrid, Stack, Text, ThemeIcon, Title } from "@mantine/core";

type SortKey = "name" | "createdAt" | "updatedAt";
type Direction = "asc" | "desc";
type SearchIndex = {
  id: string; name: string; description: string; createdAt: string; updatedAt: string;
  status: "同期済み" | "同期中" | "エラー"; vectorizer: string; store: string; chunks: string;
  sources: { name: string; type: string; path: string; files: number; lastSync: string }[];
};

const indexes: SearchIndex[] = [
  { id: "idx_01", name: "社内ナレッジ", description: "チーム文書、議事録、設計資料を横断検索", createdAt: "2026/09/01", updatedAt: "2026/09/14 09:12", status: "同期済み", vectorizer: "Multimodal Embedding v1", store: "SQLite Vector（ローカル）", chunks: "4,281", sources: [
    { name: "チーム文書", type: "Amazon S3", path: "s3://example-documents/knowledge-base", files: 248, lastSync: "今日 09:12" },
    { name: "ローカルノート", type: "Filesystem", path: "~/Documents/notes", files: 84, lastSync: "今日 09:10" },
  ] },
  { id: "idx_02", name: "プロダクト資料", description: "仕様書、リリースノート、UI 資産", createdAt: "2026/08/22", updatedAt: "2026/09/13 18:30", status: "同期中", vectorizer: "Text Embedding v3", store: "SQLite Vector（ローカル）", chunks: "1,926", sources: [{ name: "プロダクトリポジトリ", type: "Filesystem", path: "~/work/product/docs", files: 127, lastSync: "同期中" }] },
  { id: "idx_03", name: "デザインアセット", description: "ブランド画像とデザインガイドライン", createdAt: "2026/09/05", updatedAt: "2026/09/12 16:04", status: "エラー", vectorizer: "Vision Embedding v2", store: "SQLite Vector（ローカル）", chunks: "3,740", sources: [{ name: "画像アーカイブ", type: "Amazon S3", path: "s3://example-assets/design", files: 312, lastSync: "09/12 16:04" }] },
];

const statusColor = { "同期済み": "teal", "同期中": "blue", "エラー": "red" } as const;

export const Index = () => {
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [direction, setDirection] = useState<Direction>("desc");
  const [open, setOpen] = useState<string[]>(["idx_01"]);
  const sortedIndexes = useMemo(() => [...indexes].sort((a, b) => {
    const result = a[sortKey].localeCompare(b[sortKey], "ja");
    return direction === "asc" ? result : -result;
  }), [sortKey, direction]);
  const chooseSort = (key: SortKey) => {
    if (sortKey === key) setDirection(value => value === "asc" ? "desc" : "asc");
    else { setSortKey(key); setDirection(key === "name" ? "asc" : "desc"); }
  };

  return <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
    <Container size="lg">
      <Group justify="space-between" align="flex-end" mb={28}>
        <Box><Text size="sm" c="indigo" fw={700} tt="uppercase" lts={1}>Search indexes</Text><Title order={1} mt={4}>インデックス</Title><Text c="dimmed" mt={6}>ファイルソースと検索設定を管理します</Text></Box>
        <Button size="md">＋ 新しいインデックス</Button>
      </Group>
      <Card withBorder radius="md" padding="sm" mb="md"><Group justify="space-between">
        <Group gap={4}><Text size="sm" fw={600} c="dimmed" mr="xs">並び替え</Text>{([['name', '名前'], ['createdAt', '作成日'], ['updatedAt', '最終更新日']] as [SortKey, string][]).map(([key, label]) => <Button key={key} variant={sortKey === key ? "light" : "subtle"} color="gray" size="compact-sm" onClick={() => chooseSort(key)}>{label}{sortKey === key ? (direction === "asc" ? " ↑" : " ↓") : ""}</Button>)}</Group>
        <Text size="sm" c="dimmed">{indexes.length} 件のインデックス</Text>
      </Group></Card>
      <Accordion multiple value={open} onChange={setOpen} variant="separated" radius="md">
        {sortedIndexes.map(index => <Accordion.Item value={index.id} key={index.id}>
          <Accordion.Control><Group justify="space-between" wrap="nowrap" pr="md">
            <Box><Group gap="sm"><Text fw={700} size="lg">{index.name}</Text><Badge color={statusColor[index.status]} variant="light">{index.status}</Badge></Group><Text size="sm" c="dimmed" mt={3}>{index.description}</Text></Box>
            <Group gap="xl" visibleFrom="sm"><DateInfo label="作成日" value={index.createdAt} /><DateInfo label="最終更新日" value={index.updatedAt} /></Group>
          </Group></Accordion.Control>
          <Accordion.Panel><Stack gap="lg" pb="sm">
            <SimpleGrid cols={{ base: 1, sm: 3 }}><Metric label="ファイルソース" value={`${index.sources.length} 件`} /><Metric label="インデックス済みチャンク" value={index.chunks} /><Metric label="同期スケジュール" value="毎日 09:00" /></SimpleGrid>
            <Divider />
            <Group justify="space-between"><Text fw={700}>ファイルソース</Text><Button variant="subtle" size="compact-sm">＋ 追加</Button></Group>
            <Stack gap="sm">{index.sources.map(source => <Paper withBorder p="md" radius="md" key={source.name}><Group justify="space-between" align="flex-start"><Group align="flex-start" gap="sm"><ThemeIcon variant="light" color="indigo">↗</ThemeIcon><Box><Text fw={600}>{source.name}</Text><Text size="sm" c="dimmed">{source.type} · {source.path}</Text></Box></Group><Box ta="right"><Text size="sm">{source.files} ファイル</Text><Text size="xs" c="dimmed">最終同期: {source.lastSync}</Text></Box></Group></Paper>)}</Stack>
            <SimpleGrid cols={{ base: 1, sm: 2 }}><Setting label="ベクトル化" value={index.vectorizer} detail="OpenAPI エンドポイント" /><Setting label="検索 DB" value={index.store} detail="接続正常" /></SimpleGrid>
            <Group justify="flex-end"><Button variant="default">設定を編集</Button><Button>今すぐ同期</Button></Group>
          </Stack></Accordion.Panel>
        </Accordion.Item>)}
      </Accordion>
    </Container>
  </Box>;
};

function DateInfo({ label, value }: { label: string; value: string }) { return <Box><Text size="xs" c="dimmed">{label}</Text><Text size="sm">{value}</Text></Box>; }
function Metric({ label, value }: { label: string; value: string }) { return <Paper bg="gray.0" p="md" radius="md"><Text size="xs" c="dimmed">{label}</Text><Text fw={700} size="lg" mt={3}>{value}</Text></Paper>; }
function Setting({ label, value, detail }: { label: string; value: string; detail: string }) { return <Paper withBorder p="md" radius="md"><Text size="xs" c="dimmed">{label}</Text><Text fw={600} mt={3}>{value}</Text><Text size="xs" c="dimmed" mt={4}>{detail}</Text></Paper>; }
