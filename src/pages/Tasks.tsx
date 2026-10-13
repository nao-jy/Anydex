import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Badge,
  Box,
  Card,
  Container,
  Group,
  Paper,
  Stack,
  Text,
  ThemeIcon,
} from "@mantine/core";
import { commands, type IndexWithLastHistory } from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { formatDateTime } from "../components/utils";
import { PageHeader } from "../components/PageHeader";

export const Tasks = () => {
  const tasksQuery = useQuery({
    queryKey: ["activeTasks"],
    queryFn: () => unwrapCommandResult(commands.listActiveJobs()),
    refetchInterval: 5_000,
  });
  const indexesQuery = useQuery({
    queryKey: ["indexes"],
    queryFn: () => unwrapCommandResult(commands.listIndexes()),
  });
  const indexers = useMemo(
    () => buildIndexerLabels(indexesQuery.data ?? []),
    [indexesQuery.data],
  );

  const isLoading = tasksQuery.isLoading || indexesQuery.isLoading;
  const isError = tasksQuery.isError || indexesQuery.isError;
  const tasks = tasksQuery.data ?? [];

  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <Group justify="space-between" align="flex-end" mb={28}>
          <PageHeader
            pageName="タスク"
            description="現在実行中のインデックス処理を表示します"
          />
          <Badge size="lg" variant="light" color="indigo">
            実行中 {tasks.length} 件
          </Badge>
        </Group>

        {isLoading && <Message text="実行中のタスクを読み込んでいます…" />}
        {isError && (
          <Message text="タスクを取得できませんでした。" color="red" />
        )}
        {!isLoading && !isError && tasks.length === 0 && (
          <Card withBorder radius="md" p="xl">
            <Text ta="center" c="dimmed">
              現在実行中のタスクはありません。
            </Text>
          </Card>
        )}
        {!isLoading && !isError && tasks.length > 0 && (
          <Stack gap="md">
            {tasks.map((task) => {
              const labels = indexers.get(task.indexId);
              return (
                <Card key={task.id} withBorder radius="md" padding="lg">
                  <Group justify="space-between" align="flex-start">
                    <Group align="flex-start" gap="sm">
                      <ThemeIcon
                        variant="light"
                        color="indigo"
                        size="lg"
                        radius="md"
                      >
                        ↻
                      </ThemeIcon>
                      <Box>
                        <Group gap="sm">
                          <Text fw={700}>{task.jobType}</Text>
                          <Badge
                            color={task.status === "running" ? "blue" : "gray"}
                            variant="light"
                          >
                            {task.status === "running" ? "実行中" : "待機中"}
                          </Badge>
                        </Group>
                        <Text size="sm" c="dimmed" mt={4}>
                          {labels?.indexName ?? `Indexer #${task.indexId}`}
                          {labels?.sourceName ? ` / ${labels.sourceName}` : ""}
                        </Text>
                      </Box>
                    </Group>
                    <Text size="sm" c="dimmed">
                      開始:{" "}
                      {task.createdAt ? formatDateTime(task.createdAt) : "-"}
                    </Text>
                  </Group>
                  <Paper
                    p="md"
                    mt="lg"
                    radius="md"
                    bg="var(--mantine-color-default-hover)"
                  >
                    <Text size="sm" mt="xs">
                      {task.progressInfo || "進捗情報なし"}
                    </Text>
                  </Paper>
                </Card>
              );
            })}
          </Stack>
        )}
      </Container>
    </Box>
  );
};

function buildIndexerLabels(indexes: IndexWithLastHistory[]) {
  return new Map(
    indexes.map(
      ({ index }) =>
        [
          index.id,
          { indexName: index.name, sourceName: index.fileSource?.name },
        ] as const,
    ),
  );
}

function Message({ text, color = "dimmed" }: { text: string; color?: string }) {
  return (
    <Text c={color} ta="center" py="xl">
      {text}
    </Text>
  );
}
