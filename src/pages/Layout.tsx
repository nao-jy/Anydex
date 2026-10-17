import {
  ActionIcon,
  AppShell,
  Burger,
  Group,
  ScrollArea,
  Stack,
  Title,
  useMantineColorScheme,
} from "@mantine/core";
import { Outlet } from "react-router";
import { useDisclosure } from "@mantine/hooks";
import {
  IconFileText,
  IconGauge,
  IconKey,
  IconListCheck,
  IconSearch,
  IconServer,
  IconSettings,
} from "@tabler/icons-react";
import { LinksGroup } from "../components/LinksGroup";

const mockdata = [
  { label: "インデックス", icon: IconGauge, link: "/index" },
  { label: "検索", icon: IconSearch, link: "/search" },
  { label: "OAuth", icon: IconKey, link: "/oauth" },
  { label: "MCPサーバー", icon: IconServer, link: "/mcp" },
  { label: "タスク", icon: IconListCheck, link: "/tasks" },
  { label: "ライセンス", icon: IconFileText, link: "/licenses" },
  { label: "設定", icon: IconSettings, link: "/settings" },
];

export const Layout = () => {
  const { colorScheme, toggleColorScheme } = useMantineColorScheme();
  const [mobileOpened, { toggle: toggleMobile }] = useDisclosure();
  const [desktopOpened, { toggle: toggleDesktop }] = useDisclosure(true);
  const links = mockdata.map((item) => (
    <LinksGroup {...item} key={item.label} />
  ));
  return (
    <AppShell
      padding="md"
      header={{ height: 60 }}
      navbar={{
        width: 300,
        breakpoint: "sm",
        collapsed: { mobile: !mobileOpened, desktop: !desktopOpened },
      }}
    >
      <AppShell.Header>
        <Group h="100%" px="xl" justify="space-between">
          <Group gap="xl">
            <Burger
              opened={mobileOpened}
              onClick={toggleMobile}
              hiddenFrom="sm"
              size="sm"
            />
            <Burger
              opened={desktopOpened}
              onClick={toggleDesktop}
              visibleFrom="sm"
              size="sm"
            />
            <Title style={{ fontWeight: 200, userSelect: "none" }}>
              Anydex
            </Title>
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
      <AppShell.Navbar>
        <Stack gap={10} my="md">
          <ScrollArea>
            <div>{links}</div>
          </ScrollArea>
        </Stack>
      </AppShell.Navbar>
      <AppShell.Main>
        <Outlet />
      </AppShell.Main>
    </AppShell>
  );
};
