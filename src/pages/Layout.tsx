import {
  ActionIcon,
  AppShell,
  Button,
  Group,
  useMantineColorScheme,
} from "@mantine/core";
import { Link, Outlet, useLocation } from "react-router";

export const Layout = () => {
  const { colorScheme, toggleColorScheme } = useMantineColorScheme();
  const location = useLocation();
  const navButton = (to: string, label: string) => (
    <Button
      component={Link}
      to={to}
      variant={location.pathname === to ? "light" : "subtle"}
      color="gray"
      size="compact-sm"
    >
      {label}
    </Button>
  );
  return (
    <AppShell header={{ height: 60 }} padding={0}>
      <AppShell.Header>
        <Group h="100%" px="xl" justify="space-between">
          <Group gap="xl">
            <Group gap={4}>
              {navButton("/index", "インデックス")}
              {navButton("/search", "検索")}
              {navButton("/oauth", "OAuth")}
              {navButton("/mcp", "MCPサーバー")}
              {navButton("/tasks", "タスク")}
              {navButton("/settings", "設定")}
            </Group>
          </Group>
          <Group gap="sm">
            <ActionIcon
              variant="default"
              size="lg"
              aria-label="ダークモードを切り替え"
              onClick={() => toggleColorScheme()}
            >
              {colorScheme === "dark" ? "☀" : "◐"}
            </ActionIcon>
          </Group>
        </Group>
      </AppShell.Header>
      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  );
};
