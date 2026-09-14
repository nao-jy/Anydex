import { useState } from "react";
import { Badge, Box, Button, Card, Checkbox, Container, Divider, Group, Paper, Select, Stack, Text, TextInput, ThemeIcon, Title } from "@mantine/core";

const results = [
  { title: "2026年度 プロダクトロードマップ", source: "社内ナレッジ / チーム文書", type: "PDF", score: "96%", excerpt: "第 3 四半期では検索基盤の刷新を行い、マルチモーダルコンテンツへの対応を段階的に開始します。…", updated: "今日 09:12" },
  { title: "検索体験の設計方針", source: "プロダクト資料 / プロダクトリポジトリ", type: "Markdown", score: "91%", excerpt: "検索結果には出所と関連度を明示し、利用者が原文へすぐ遷移できる状態を保つ。ベクトル検索とメタデータフィルタを…", updated: "昨日 18:30" },
  { title: "検索画面ワイヤーフレーム", source: "デザインアセット / 画像アーカイブ", type: "PNG", score: "87%", excerpt: "検索ボックス、インデックス選択、結果一覧を含む初期画面のデザイン案。…", updated: "09/12 16:04" },
];

export const Search = () => {
  const [query, setQuery] = useState("検索基盤のロードマップ");
  const [hasSearched, setHasSearched] = useState(true);
  return <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
    <Container size="lg">
      <Box mb={28}><Text size="sm" c="indigo" fw={700} tt="uppercase" lts={1}>Semantic search</Text><Title order={1} mt={4}>検索</Title><Text c="dimmed" mt={6}>インデックス横断で、意味に近いコンテンツを探します</Text></Box>
      <Card withBorder radius="md" padding="lg" mb="xl">
        <Group align="flex-end">
          <TextInput label="何を探していますか？" placeholder="質問やキーワードを入力" value={query} onChange={(event) => setQuery(event.currentTarget.value)} onKeyDown={(event) => event.key === "Enter" && setHasSearched(true)} style={{ flex: 1 }} size="md" />
          <Button size="md" onClick={() => setHasSearched(true)}>検索</Button>
        </Group>
        <Group mt="md" justify="space-between"><Select label="検索対象" defaultValue="all" data={[{ value: "all", label: "すべてのインデックス" }, { value: "knowledge", label: "社内ナレッジ" }, { value: "product", label: "プロダクト資料" }]} w={240} /><Group mt={24} gap="md"><Checkbox label="画像を含める" defaultChecked /><Checkbox label="更新日の新しい順を優先" /></Group></Group>
      </Card>
      {hasSearched && <Group align="flex-start" gap="xl">
        <Box w={190} visibleFrom="sm"><Text fw={700} size="sm" mb="sm">絞り込み</Text><Paper withBorder p="md" radius="md"><Text size="sm" fw={600} mb="xs">コンテンツ種別</Text><Stack gap="xs"><Checkbox label="テキスト" defaultChecked /><Checkbox label="画像" defaultChecked /><Checkbox label="PDF" defaultChecked /></Stack><Divider my="md" /><Text size="sm" fw={600} mb="xs">インデックス</Text><Stack gap="xs"><Checkbox label="社内ナレッジ" defaultChecked /><Checkbox label="プロダクト資料" defaultChecked /><Checkbox label="デザインアセット" defaultChecked /></Stack></Paper></Box>
        <Box style={{ flex: 1 }}><Group justify="space-between" mb="sm"><Text c="dimmed" size="sm"><Text component="span" fw={700} c="var(--mantine-color-text)">3 件</Text>の検索結果</Text><Select size="xs" w={150} defaultValue="relevance" data={[{ value: "relevance", label: "関連度順" }, { value: "updated", label: "更新日順" }]} /></Group>
          <Stack gap="sm">{results.map((result, index) => <Paper withBorder radius="md" p="lg" key={result.title}>
            <Group justify="space-between" align="flex-start" wrap="nowrap"><Group align="flex-start" gap="sm"><ThemeIcon color={index === 2 ? "grape" : "indigo"} variant="light" size="lg">{index === 2 ? "▧" : "⌕"}</ThemeIcon><Box><Group gap="sm"><Text fw={700}>{result.title}</Text><Badge variant="light" color="gray">{result.type}</Badge></Group><Text size="sm" c="dimmed" mt={3}>{result.source}</Text></Box></Group><Badge color="teal" variant="light">関連度 {result.score}</Badge></Group>
            <Text size="sm" mt="md" lh={1.7}>{result.excerpt}</Text><Group justify="space-between" mt="md"><Text size="xs" c="dimmed">最終更新: {result.updated}</Text><Button variant="subtle" size="compact-sm">ファイルを開く →</Button></Group>
          </Paper>)}</Stack>
        </Box>
      </Group>}
    </Container>
  </Box>;
};
