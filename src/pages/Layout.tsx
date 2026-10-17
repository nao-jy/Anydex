import {
  ActionIcon,
  AppShell,
  Burger,
  Group,
  Select,
  ScrollArea,
  Stack,
  Title,
  useMantineColorScheme,
} from "@mantine/core";
import { Outlet } from "react-router";
import { useDisclosure } from "@mantine/hooks";
import {
  IconGauge,
  IconKey,
  IconListCheck,
  IconSearch,
  IconServer,
  IconSettings,
} from "@tabler/icons-react";
import { LinksGroup } from "../components/LinksGroup";
import { useTranslation } from "react-i18next";
import i18n from "../i18n";

export const Layout = () => {
  const { t } = useTranslation();
  const { colorScheme, toggleColorScheme } = useMantineColorScheme();
  const [mobileOpened, { toggle: toggleMobile }] = useDisclosure();
  const [desktopOpened, { toggle: toggleDesktop }] = useDisclosure(true);
  const mockdata = [
    { label: t("インデックス"), icon: IconGauge, link: "/index" },
    { label: t("検索"), icon: IconSearch, link: "/search" },
    { label: "OAuth", icon: IconKey, link: "/oauth" },
    { label: t("MCPサーバー"), icon: IconServer, link: "/mcp" },
    { label: t("タスク"), icon: IconListCheck, link: "/tasks" },
    { label: t("設定"), icon: IconSettings, link: "/settings" },
  ];
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
            <Select
              aria-label={t("言語")}
              value={i18n.language.startsWith("ja") ? "ja" : "en"}
              data={[
                { label: "日本語", value: "ja" },
                { label: "English", value: "en" },
              ]}
              onChange={(language) => {
                if (language) void i18n.changeLanguage(language);
              }}
              w={112}
              size="sm"
              allowDeselect={false}
            />
            <ActionIcon
              variant="default"
              size="lg"
              aria-label={t("ダークモードを切り替え")}
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
