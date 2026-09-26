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
  Title,
} from "@mantine/core";
import { commands } from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { formatDateTime } from "../components/utils";

export const Mcp = () => {
  const queryClient = useQueryClient();
  const [selectedPort, setSelectedPort] = useState<string>("8080");
  const {
    data: servers = [],
    isLoading: isLoadingServers,
    isError: isServerError,
  } = useQuery({
    queryKey: ["mcpServers"],
    queryFn: () => unwrapCommandResult(commands.listMcpServers()),
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
    mutationFn: ({ indexId, enabled }: { indexId: number; enabled: boolean }) =>
      unwrapCommandResult(commands.setIndexMcpServer(indexId, enabled)),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["indexes"] });
      void queryClient.invalidateQueries({ queryKey: ["mcpServers"] });
    },
  });
  const setPort = useMutation({
    mutationFn: (port: number) => commands.setMcpServerPort(port),
    onSuccess: () =>
      void queryClient.invalidateQueries({ queryKey: ["mcpServers"] }),
  });
  const setServerRunning = useMutation({
    mutationFn: (running: boolean) => commands.setMcpServerRunning(running),
    onSuccess: () =>
      void queryClient.invalidateQueries({ queryKey: ["mcpServers"] }),
  });

  const server = servers[0];
  useEffect(() => {
    if (server) setSelectedPort(server.port.toString());
  }, [server]);

  const isLoading = isLoadingServers || isLoadingIndexes;
  const isError = isServerError || isIndexError;

  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <Box mb={28}>
          <Text size="sm" c="indigo" fw={700} tt="uppercase" lts={1}>
            Local integration
          </Text>
          <Title order={1} mt={4}>
            MCPサーバー
          </Title>
          <Text c="dimmed" mt={6}>
            MCP から公開するインデックスとローカルサーバーを管理します
          </Text>
        </Box>

        {isLoading && (
          <LoadingMessage text="MCP サーバーの状態を読み込んでいます…" />
        )}
        {isError && (
          <ErrorMessage text="MCP サーバーの情報を取得できませんでした。" />
        )}

        {!isLoading && !isError && server && (
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
                      <Text fw={700}>{server.name}</Text>
                      <Badge
                        color={server.status === "running" ? "teal" : "gray"}
                        variant="light"
                      >
                        {server.status === "running" ? "稼働中" : "停止中"}
                      </Badge>
                    </Group>
                    <Text size="sm" c="dimmed" mt={4}>
                      {server.host}:{server.port}
                    </Text>
                  </Box>
                </Group>
                <Text size="sm" c="dimmed">
                  起動:{" "}
                  {server.startedAt ? formatDateTime(server.startedAt) : "-"}
                </Text>
              </Group>
              <SimpleGrid cols={{ base: 1, sm: 3 }} mt="lg">
                <Metric
                  label="公開中のインデックス"
                  value={`${server.enabledIndexCount} 件`}
                />
                <Metric label="接続先" value={server.host} />
                <Metric
                  label="ステータス"
                  value={server.status === "running" ? "稼働中" : "停止中"}
                />
              </SimpleGrid>
              <Group justify="flex-end" mt="lg">
                <Button
                  color={server.status === "running" ? "red" : "teal"}
                  variant={server.status === "running" ? "light" : "filled"}
                  onClick={() =>
                    setServerRunning.mutate(server.status !== "running")
                  }
                  loading={setServerRunning.isPending}
                >
                  {server.status === "running"
                    ? "サーバーを停止"
                    : "サーバーを起動"}
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
                  disabled={server.status === "running"}
                  w={180}
                />
                <Button
                  onClick={() =>
                    selectedPort && setPort.mutate(Number(selectedPort))
                  }
                  loading={setPort.isPending}
                  disabled={
                    !selectedPort || selectedPort === server.port.toString()
                  }
                >
                  ポートを保存
                </Button>
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
                {indexes.map((index) => (
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
