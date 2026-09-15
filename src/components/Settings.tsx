import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Badge,
  Box,
  Button,
  Card,
  Code,
  Container,
  Group,
  Paper,
  Stack,
  Text,
  ThemeIcon,
  Title,
} from "@mantine/core";
import { commands } from "../bindings";

export const Settings = () => {
  const queryClient = useQueryClient();
  const settingsQuery = useQuery({
    queryKey: ["applicationSettings"],
    queryFn: commands.getApplicationSettings,
  });
  const reloadPlugins = useMutation({
    mutationFn: commands.reloadPlugins,
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ["applicationSettings"] }),
  });
  const settings = settingsQuery.data;

  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <Box mb={28}>
          <Text size="sm" c="indigo" fw={700} tt="uppercase" lts={1}>
            Application
          </Text>
          <Title order={1} mt={4}>
            設定
          </Title>
          <Text c="dimmed" mt={6}>
            ローカルデータとプラグインの配置を確認します
          </Text>
        </Box>

        {settingsQuery.isLoading && <Message text="設定を読み込んでいます…" />}
        {settingsQuery.isError && <Message text="設定を取得できませんでした。" color="red" />}
        {settings && (
          <Stack gap="lg">
            <Card withBorder radius="md" padding="lg">
              <Text fw={700}>ローカルデータ</Text>
              <Stack gap="md" mt="md">
                <PathRow label="SQLite データベース" path={settings.databasePath} />
                <PathRow label="ログディレクトリ" path={settings.logDirectory} />
              </Stack>
            </Card>

            <Card withBorder radius="md" padding="lg">
              <Group justify="space-between" align="flex-start">
                <Box>
                  <Text fw={700}>プラグイン</Text>
                  <Text size="sm" c="dimmed" mt={4}>
                    次回の実装では、この場所から抽出器・ベクトルストア・ベクトル化プラグインを読み込みます。
                  </Text>
                </Box>
                <Button
                  variant="light"
                  onClick={() => reloadPlugins.mutate()}
                  loading={reloadPlugins.isPending}
                >
                  プラグインを再読み込み
                </Button>
              </Group>
              {reloadPlugins.data && (
                <Text size="sm" c="teal" mt="md">
                  {reloadPlugins.data.discoveredPlugins} 件を検出しました（
                  {formatDateTime(reloadPlugins.data.reloadedAt)}）。
                </Text>
              )}
              <Stack gap="xs" mt="lg">
                {settings.pluginDirectories.map((directory) => (
                  <Paper key={directory.name} withBorder p="md" radius="md">
                    <Group justify="space-between" align="flex-start" wrap="nowrap">
                      <Group align="flex-start" gap="sm">
                        <ThemeIcon variant="light" color="indigo">⌘</ThemeIcon>
                        <Box>
                          <Text fw={600}>{directory.name}</Text>
                          <Code block mt={6}>{directory.path}</Code>
                        </Box>
                      </Group>
                      <Badge variant="light" color="gray">
                        {directory.pluginCount} 件
                      </Badge>
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

function PathRow({ label, path }: { label: string; path: string }) {
  return (
    <Paper p="md" radius="md" bg="var(--mantine-color-default-hover)">
      <Text size="sm" fw={600}>{label}</Text>
      <Code block mt={6}>{path}</Code>
    </Paper>
  );
}

function Message({ text, color = "dimmed" }: { text: string; color?: string }) {
  return <Text c={color} ta="center" py="xl">{text}</Text>;
}

function formatDateTime(value: string) {
  return new Intl.DateTimeFormat("ja-JP", {
    dateStyle: "short",
    timeStyle: "short",
    timeZone: "Asia/Tokyo",
  }).format(new Date(value));
}
