import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Badge,
  Box,
  Button,
  Card,
  Container,
  Group,
  Loader,
  MultiSelect,
  Paper,
  Select,
  Stack,
  Text,
  TextInput,
  ThemeIcon,
  Title,
} from "@mantine/core";
import { commands } from "../bindings";

export const Search = () => {
  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const [hasSearched, setHasSearched] = useState(false);
  const [searchRequest, setSearchRequest] = useState(0);
  const { data: indexes = [] } = useQuery({
    queryKey: ["indexes"],
    queryFn: commands.listIndexes,
  });
  const {
    data: results = [],
    isLoading,
    isError,
  } = useQuery({
    queryKey: ["search", submittedQuery, searchRequest],
    queryFn: () => commands.searchMock(submittedQuery),
    enabled: hasSearched,
  });
  const runSearch = () => {
    setSubmittedQuery(query);
    setSearchRequest((value) => value + 1);
    setHasSearched(true);
  };
  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <Box mb={28}>
          <Text size="sm" c="indigo" fw={700} tt="uppercase" lts={1}>
            Semantic search
          </Text>
          <Title order={1} mt={4}>
            検索
          </Title>
          <Text c="dimmed" mt={6}>
            インデックス横断で、意味に近いコンテンツを探します
          </Text>
        </Box>
        <Card withBorder radius="md" padding="lg" mb="xl">
          <Group align="flex-end">
            <TextInput
              label="何を探していますか？"
              placeholder="質問やキーワードを入力"
              value={query}
              onChange={(event) => setQuery(event.currentTarget.value)}
              onKeyDown={(event) => event.key === "Enter" && runSearch()}
              style={{ flex: 1 }}
              size="md"
            />
            <Button size="md" onClick={runSearch}>
              検索
            </Button>
          </Group>
          <Group mt="md" gap="md">
            <Select
              label="検索対象"
              data={indexes.map((index) => ({
                value: index.id,
                label: index.name,
              }))}
              w={240}
            />
            <MultiSelect
              label="コンテンツ種別"
              data={[
                { value: "text", label: "テキスト" },
                { value: "image", label: "画像" },
                { value: "others", label: "その他" },
              ]}
              w={180}
            />
          </Group>
        </Card>
        {hasSearched && (
          <Group align="flex-start" gap="xl">
            <Box style={{ flex: 1 }}>
              {isLoading && (
                <Stack align="center" gap="sm" py="xl">
                  <Loader color="indigo" />
                  <Text c="dimmed">検索しています…</Text>
                </Stack>
              )}
              {isError && (
                <Text c="red" ta="center" py="xl">
                  検索結果を取得できませんでした。
                </Text>
              )}
              <Stack gap="sm">
                {results.map((result, index) => (
                  <Paper withBorder radius="md" p="lg" key={result.title}>
                    <Group
                      justify="space-between"
                      align="flex-start"
                      wrap="nowrap"
                    >
                      <Group align="flex-start" gap="sm">
                        <ThemeIcon
                          color={index === 2 ? "grape" : "indigo"}
                          variant="light"
                          size="lg"
                        >
                          {index === 2 ? "▧" : "⌕"}
                        </ThemeIcon>
                        <Box>
                          <Group gap="sm">
                            <Text fw={700}>{result.title}</Text>
                            <Badge variant="light" color="gray">
                              {result.contentType}
                            </Badge>
                          </Group>
                          <Text size="sm" c="dimmed" mt={3}>
                            {result.source}
                          </Text>
                        </Box>
                      </Group>
                      <Badge color="teal" variant="light">
                        関連度 {result.score}
                      </Badge>
                    </Group>
                    <Text size="sm" mt="md" lh={1.7}>
                      {result.excerpt}
                    </Text>
                    <Group justify="space-between" mt="md">
                      <Text size="xs" c="dimmed">
                        最終更新: {result.updatedAt}
                      </Text>
                      <Button variant="subtle" size="compact-sm">
                        ファイルを開く →
                      </Button>
                    </Group>
                  </Paper>
                ))}
              </Stack>
            </Box>
          </Group>
        )}
      </Container>
    </Box>
  );
};
