import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  ActionIcon,
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
} from "@mantine/core";
import { commands } from "../bindings";
import { openPath } from "@tauri-apps/plugin-opener";
import { unwrapCommandResult } from "../tauri";
import { PageHeader } from "../components/PageHeader";

export const Settings = () => {
  const queryClient = useQueryClient();
  const settingsQuery = useQuery({
    queryKey: ["applicationSettings"],
    queryFn: commands.getApplicationSettings,
  });
  const getPluginInfo = useQuery({
    queryKey: ["getPluginInfo"],
    queryFn: async () => await unwrapCommandResult(commands.getPluginInfo()),
  });
  const reloadPlugins = useMutation({
    mutationKey: ["reloadPlugins"],
    mutationFn: async () => {
      await unwrapCommandResult(commands.reloadPlugins());
      queryClient.invalidateQueries({ queryKey: ["getPluginInfo"] });
    },
  });
  const settings = settingsQuery.data;

  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <PageHeader pageName="設定" description="各種設定を閲覧・変更します" />

        {settingsQuery.isLoading && <Message text="設定を読み込んでいます…" />}
        {settingsQuery.isError && (
          <Message text="設定を取得できませんでした。" color="red" />
        )}
        {settings && (
          <Stack gap="lg">
            <Card withBorder radius="md" padding="lg">
              <Text fw={700}>ローカルデータ</Text>
              <Stack gap="md" mt="md">
                <PathRow
                  label="SQLite データベース"
                  path={settings.databasePath}
                />
                <PathRow
                  label="ログディレクトリ"
                  path={settings.logDirectory}
                />
              </Stack>
            </Card>

            <Card withBorder radius="md" padding="lg">
              <Group justify="space-between" align="flex-start">
                <Text fw={700}>プラグイン</Text>
                <Button
                  variant="light"
                  onClick={() => reloadPlugins.mutate()}
                  loading={getPluginInfo.isPending || reloadPlugins.isPending}
                >
                  プラグインを再読み込み
                </Button>
              </Group>
              <Stack gap="xs" mt="lg">
                {getPluginInfo.data?.map((info) => (
                  <Paper key={info.pluginType} withBorder p="md" radius="md">
                    <Group
                      justify="space-between"
                      align="flex-start"
                      wrap="nowrap"
                    >
                      <Group align="flex-start" gap="sm">
                        <ThemeIcon variant="light" color="indigo">
                          ⌘
                        </ThemeIcon>
                        <Text fw={600}>{info.pluginType}</Text>
                      </Group>
                      <Badge variant="light" color="gray">
                        {info.plugins.length} 件
                      </Badge>
                    </Group>
                    <Group>
                      <ActionIcon
                        onClick={async () => {
                          await openPath(info.path);
                        }}
                        variant="subtle"
                      >
                        📂
                      </ActionIcon>
                      <Code block mt={6}>
                        {info.path}
                      </Code>
                    </Group>
                    {info.plugins.map((v) => (
                      <Badge key={v}>{v}</Badge>
                    ))}
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
      <Text size="sm" fw={600}>
        {label}
      </Text>
      <Code block mt={6}>
        {path}
      </Code>
    </Paper>
  );
}

function Message({ text, color = "dimmed" }: { text: string; color?: string }) {
  return (
    <Text c={color} ta="center" py="xl">
      {text}
    </Text>
  );
}
