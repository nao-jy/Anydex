import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Badge,
  Box,
  Card,
  Container,
  Group,
  Paper,
  Progress,
  Stack,
  Text,
  ThemeIcon,
  Title,
} from "@mantine/core";
import { commands, type Index } from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { formatDateTime } from "../components/utils";

export const Tasks = () => {
  const tasksQuery = useQuery({
    queryKey: ["activeTasks"],
    queryFn: () => unwrapCommandResult(commands.listActiveTasks()),
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
          <Box>
            <Text size="sm" c="indigo" fw={700} tt="uppercase" lts={1}>
              Background jobs
            </Text>
            <Title order={1} mt={4}>
              タスク
            </Title>
            <Text c="dimmed" mt={6}>
              現在実行中のインデックス処理を表示します
            </Text>
          </Box>
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
              const state = asObject(task.state);
              const progress = numberValue(state.progress);
              const labels = indexers.get(task.indexerId);
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
                          <Text fw={700}>{task.taskType}</Text>
                          <Badge color="blue" variant="light">
                            実行中
                          </Badge>
                        </Group>
                        <Text size="sm" c="dimmed" mt={4}>
                          {labels?.indexName ?? `Indexer #${task.indexerId}`}
                          {labels?.sourceName ? ` / ${labels.sourceName}` : ""}
                        </Text>
                      </Box>
                    </Group>
                    <Text size="sm" c="dimmed">
                      開始:{" "}
                      {task.startedAt ? formatDateTime(task.startedAt) : "-"}
                    </Text>
                  </Group>
                  <Paper
                    p="md"
                    mt="lg"
                    radius="md"
                    bg="var(--mantine-color-default-hover)"
                  >
                    <Group justify="space-between">
                      <Text size="sm" fw={600}>
                        {stringValue(state.phase) ?? "処理中"}
                      </Text>
                      <Text size="sm" c="dimmed">
                        {progress == null ? "進捗を集計中" : `${progress}%`}
                      </Text>
                    </Group>
                    {progress != null && <Progress value={progress} mt="sm" />}
                    <Text size="xs" c="dimmed" mt="sm">
                      {formatFileProgress(state)}
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

function buildIndexerLabels(indexes: Index[]) {
  return new Map(
    indexes.flatMap((index) =>
      index.indexers.map(
        (indexer) =>
          [
            indexer.id,
            { indexName: index.name, sourceName: indexer.fileSource?.name },
          ] as const,
      ),
    ),
  );
}

function asObject(value: unknown): Record<string, unknown> {
  return typeof value === "object" && value != null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function numberValue(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function stringValue(value: unknown) {
  return typeof value === "string" ? value : null;
}

function formatFileProgress(state: Record<string, unknown>) {
  const processed = numberValue(state.processedFiles);
  const total = numberValue(state.totalFiles);
  return processed != null && total != null
    ? `${processed} / ${total} ファイルを処理済み`
    : "ファイルを処理しています";
}

function Message({ text, color = "dimmed" }: { text: string; color?: string }) {
  return (
    <Text c={color} ta="center" py="xl">
      {text}
    </Text>
  );
}
