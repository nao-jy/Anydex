import { useQuery } from "@tanstack/react-query";
import {
  ActionIcon,
  Badge,
  Box,
  Card,
  Code,
  Container,
  Button,
  Group,
  Paper,
  Stack,
  Text,
  ThemeIcon,
} from "@mantine/core";
import { Link } from "react-router";
import { commands } from "../bindings";
import { openPath } from "@tauri-apps/plugin-opener";
import { unwrapCommandResult } from "../tauri";
import { PageHeader } from "../components/PageHeader";

export const Settings = () => {
  const settingsQuery = useQuery({
    queryKey: ["applicationSettings"],
    queryFn: commands.getApplicationSettings,
  });
  const getPluginInfo = useQuery({
    queryKey: ["getPluginInfo"],
    queryFn: async () => await unwrapCommandResult(commands.getPluginInfo()),
  });
  const settings = settingsQuery.data;

  return (
    <Box bg="var(--mantine-color-body)" py={20}>
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
              <Text fw={700}>プラグイン</Text>
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
                    <Group>
                      {info.plugins.map((v) => (
                        <Badge key={v}>{v}</Badge>
                      ))}
                    </Group>
                  </Paper>
                ))}
              </Stack>
            </Card>
          </Stack>
        )}

        <Card withBorder radius="md" padding="lg" mt="lg">
          <Text fw={700}>オープンソースライセンス</Text>
          <Text size="sm" c="dimmed" mt="xs">
            アプリで利用している依存ライブラリのライセンス情報を確認できます。
          </Text>
          <Button component={Link} to="/licenses" mt="md" variant="light">
            ライセンス一覧を見る
          </Button>
        </Card>
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
