import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Accordion, Badge, Box, Button, Card, Container, Divider, Group, Paper, SimpleGrid, Stack, Text, ThemeIcon, Title } from "@mantine/core";
import { commands } from "../bindings";

type SortKey = "name" | "createdAt" | "updatedAt";
type Direction = "asc" | "desc";
const statusColor: Record<string, "teal" | "blue" | "red" | "gray"> = { "同期済み": "teal", "同期中": "blue", "エラー": "red" };

export const Index = () => {
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [direction, setDirection] = useState<Direction>("desc");
  const [open, setOpen] = useState<string[]>(["idx_01"]);
  const { data: indexes = [], isLoading, isError } = useQuery({ queryKey: ["indexes"], queryFn: commands.listIndexes });
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
      {isLoading && <Text c="dimmed" ta="center" py="xl">インデックスを読み込んでいます…</Text>}
      {isError && <Text c="red" ta="center" py="xl">インデックスを取得できませんでした。</Text>}
      {!isLoading && !isError && <Accordion multiple value={open} onChange={setOpen} variant="separated" radius="md">
        {sortedIndexes.map(index => <Accordion.Item value={index.id} key={index.id}>
          <Accordion.Control><Group justify="space-between" wrap="nowrap" pr="md">
            <Box><Group gap="sm"><Text fw={700} size="lg">{index.name}</Text><Badge color={statusColor[index.status] ?? "gray"} variant="light">{index.status}</Badge></Group><Text size="sm" c="dimmed" mt={3}>{index.description}</Text></Box>
            <Group gap="xl" visibleFrom="sm"><DateInfo label="作成日" value={index.createdAt} /><DateInfo label="最終更新日" value={index.updatedAt} /></Group>
          </Group></Accordion.Control>
          <Accordion.Panel><Stack gap="lg" pb="sm">
            <SimpleGrid cols={{ base: 1, sm: 3 }}><Metric label="ファイルソース" value={`${index.sources.length} 件`} /><Metric label="インデックス済みチャンク" value={index.chunks} /><Metric label="同期スケジュール" value="毎日 09:00" /></SimpleGrid>
            <Divider />
            <Group justify="space-between"><Text fw={700}>ファイルソース</Text><Button variant="subtle" size="compact-sm">＋ 追加</Button></Group>
            <Stack gap="sm">{index.sources.map(source => <Paper withBorder p="md" radius="md" key={source.name}><Group justify="space-between" align="flex-start"><Group align="flex-start" gap="sm"><ThemeIcon variant="light" color="indigo">↗</ThemeIcon><Box><Text fw={600}>{source.name}</Text><Text size="sm" c="dimmed">{source.sourceType} · {source.path}</Text></Box></Group><Box ta="right"><Text size="sm">{source.files} ファイル</Text><Text size="xs" c="dimmed">最終同期: {source.lastSync}</Text></Box></Group></Paper>)}</Stack>
            <SimpleGrid cols={{ base: 1, sm: 2 }}><Setting label="ベクトル化" value={index.vectorizer} detail="OpenAPI エンドポイント" /><Setting label="検索 DB" value={index.store} detail="接続正常" /></SimpleGrid>
            <Group justify="flex-end"><Button variant="default">設定を編集</Button><Button>今すぐ同期</Button></Group>
          </Stack></Accordion.Panel>
        </Accordion.Item>)}
      </Accordion>}
    </Container>
  </Box>;
};

function DateInfo({ label, value }: { label: string; value: string }) { return <Box><Text size="xs" c="dimmed">{label}</Text><Text size="sm">{value}</Text></Box>; }
function Metric({ label, value }: { label: string; value: string }) { return <Paper bg="gray.0" p="md" radius="md"><Text size="xs" c="dimmed">{label}</Text><Text fw={700} size="lg" mt={3}>{value}</Text></Paper>; }
function Setting({ label, value, detail }: { label: string; value: string; detail: string }) { return <Paper withBorder p="md" radius="md"><Text size="xs" c="dimmed">{label}</Text><Text fw={600} mt={3}>{value}</Text><Text size="xs" c="dimmed" mt={4}>{detail}</Text></Paper>; }
