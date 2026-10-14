import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Badge,
  Box,
  Button,
  Card,
  Container,
  Group,
  NumberInput,
  Paper,
  SimpleGrid,
  Stack,
  Switch,
  Text,
  ThemeIcon,
} from "@mantine/core";
import { commands } from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { formatDateTime } from "../components/utils";
import { PageHeader } from "../components/PageHeader";

export const Mcp = () => {
  const queryClient = useQueryClient();
  const [selectedPort, setSelectedPort] = useState<string>("8080");
  const {
    data: server = null,
    isLoading: isLoadingServer,
    isError: isServerError,
  } = useQuery({
    queryKey: ["mcpServer"],
    queryFn: () => unwrapCommandResult(commands.getMcpServerState()),
  });
  const {
    data: indexes = [],
    isLoading: isLoadingIndexes,
    isError: isIndexError,
  } = useQuery({
    queryKey: ["indexes"],
    queryFn: () => unwrapCommandResult(commands.listIndexes()),
  });

  const setIndexEnabled = useMutation({
    mutationFn: ({ indexId, enabled }: { indexId: string; enabled: boolean }) =>
      unwrapCommandResult(commands.setIndexMcpServer(indexId, enabled)),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["indexes"] });
      void queryClient.invalidateQueries({ queryKey: ["mcpServer"] });
    },
  });
  const setServerRunning = useMutation({
    mutationFn: (running: boolean) =>
      commands.setMcpServerStatus(running, Number(selectedPort)),
    onSuccess: () =>
      void queryClient.invalidateQueries({ queryKey: ["mcpServer"] }),
  });

  useEffect(() => {
    if (server) setSelectedPort(server.port.toString());
  }, [server]);

  const isRunning = server !== null;
  const isLoading = isLoadingServer || isLoadingIndexes;
  const isError = isServerError || isIndexError;

  return (
    <Box bg="var(--mantine-color-body)" py={20}>
      <Container size="lg">
        <PageHeader
          pageName="MCP & ファイルサーバー"
          description="MCP から公開するインデックスとローカルサーバーを管理します"
        />

        {isLoading && (
          <LoadingMessage text="MCP サーバーの状態を読み込んでいます…" />
        )}
        {isError && (
          <ErrorMessage text="MCP サーバーの情報を取得できませんでした。" />
        )}

        {!isLoading && !isError && (
          <Stack gap="lg">
            <Card withBorder radius="md" padding="lg">
              <Group justify="space-between" align="flex-start">
                <Group gap="sm">
                  <ThemeIcon
                    size="lg"
                    radius="md"
                    variant="light"
                    color="indigo"
                  >
                    ↔
                  </ThemeIcon>
                  <Box>
                    <Group gap="sm">
                      <Badge
                        color={isRunning ? "teal" : "gray"}
                        variant="light"
                      >
                        {isRunning ? "稼働中" : "停止中"}
                      </Badge>
                    </Group>
                    {server && (
                      <Text size="sm" c="dimmed" mt={4}>
                        {`localhost:${server.port}`}
                      </Text>
                    )}
                  </Box>
                </Group>
                <Text size="sm" c="dimmed">
                  起動:{" "}
                  {server?.startedAt ? formatDateTime(server.startedAt) : "-"}
                </Text>
              </Group>
              <SimpleGrid cols={{ base: 1, sm: 2 }} mt="lg">
                <Metric
                  label="公開中のインデックス"
                  value={`${indexes.filter((v) => v.index.useMcpServer).length} 件`}
                />
                <Metric
                  label="ステータス"
                  value={isRunning ? "稼働中" : "停止中"}
                />
              </SimpleGrid>
              <Group justify="flex-end" mt="lg">
                <Button
                  color={isRunning ? "red" : "teal"}
                  variant={isRunning ? "light" : "filled"}
                  onClick={() => setServerRunning.mutate(!isRunning)}
                  loading={setServerRunning.isPending}
                >
                  {isRunning ? "サーバーを停止" : "サーバーを起動"}
                </Button>
              </Group>
            </Card>

            <Card withBorder radius="md" padding="lg">
              <Text fw={700}>待受ポート</Text>
              <Text size="sm" c="dimmed" mt={4}>
                MCP クライアントが接続するローカルポートを選択します。
              </Text>
              <Group mt="md" align="flex-end">
                <NumberInput
                  label="ポート"
                  value={selectedPort}
                  onChange={(v) => {
                    setSelectedPort(v.toString());
                  }}
                  disabled={isRunning}
                  w={180}
                />
              </Group>
            </Card>

            <Card withBorder radius="md" padding="lg">
              <Group justify="space-between" mb="xs">
                <Text fw={700}>公開するインデックス</Text>
                <Text size="sm" c="dimmed">
                  有効なインデックスだけを MCP から検索できます
                </Text>
              </Group>
              <Stack gap="xs" mt="md">
                {indexes.map(({ index }) => (
                  <Paper key={index.id} withBorder p="md" radius="md">
                    <Group justify="space-between" wrap="nowrap">
                      <Box>
                        <Text fw={600}>{index.name}</Text>
                        <Text size="sm" c="dimmed" mt={3}>
                          {index.description}
                        </Text>
                      </Box>
                      <Switch
                        checked={index.useMcpServer}
                        onChange={(event) =>
                          setIndexEnabled.mutate({
                            indexId: index.id,
                            enabled: event.currentTarget.checked,
                          })
                        }
                        disabled={setIndexEnabled.isPending}
                        aria-label={`${index.name} を MCP から公開`}
                        label={index.useMcpServer ? "公開中" : "非公開"}
                      />
                    </Group>
                  </Paper>
                ))}
              </Stack>
            </Card>
          </Stack>
        )}
      </Container>
    </Box>
  );
};

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <Paper p="md" radius="md" bg="var(--mantine-color-default-hover)">
      <Text size="xs" c="dimmed">
        {label}
      </Text>
      <Text fw={700} mt={3}>
        {value}
      </Text>
    </Paper>
  );
}

function LoadingMessage({ text }: { text: string }) {
  return (
    <Text c="dimmed" ta="center" py="xl">
      {text}
    </Text>
  );
}

function ErrorMessage({ text }: { text: string }) {
  return (
    <Text c="red" ta="center" py="xl">
      {text}
    </Text>
  );
}
