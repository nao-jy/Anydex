import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Accordion,
  Badge,
  Box,
  Card,
  Container,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { PageHeader } from "../components/PageHeader";

interface DependencyLicense {
  ecosystem: string;
  name: string;
  version: string;
  license: string;
  licenseText: string;
}

export const Licenses = () => {
  const [search, setSearch] = useState("");
  const licensesQuery = useQuery({
    queryKey: ["dependencyLicenses"],
    queryFn: async () => {
      const response = await fetch("/licenses.json");
      if (!response.ok) {
        throw new Error(`ライセンス一覧の取得に失敗しました: ${response.status}`);
      }
      return (await response.json()) as DependencyLicense[];
    },
  });
  const licenses = licensesQuery.data ?? [];
  const filteredLicenses = useMemo(() => {
    const query = search.trim().toLocaleLowerCase();
    if (!query) {
      return licenses;
    }
    return licenses.filter((item) =>
      [item.name, item.version, item.license, item.ecosystem]
        .join(" ")
        .toLocaleLowerCase()
        .includes(query),
    );
  }, [licenses, search]);

  return (
    <Box bg="var(--mantine-color-body)" py={20}>
      <Container size="lg">
        <PageHeader
          pageName="オープンソースライセンス"
          description="アプリで利用している npm パッケージと Rust クレートのライセンス表記です。"
        />

        {licensesQuery.isLoading && (
          <Message text="ライセンス一覧を読み込んでいます…" />
        )}
        {licensesQuery.isError && (
          <Message text="ライセンス一覧を取得できませんでした。" color="red" />
        )}
        {licensesQuery.data && (
          <Stack gap="md">
            <TextInput
              label="依存ライブラリを検索"
              placeholder="名前、バージョン、ライセンス"
              value={search}
              onChange={(event) => setSearch(event.currentTarget.value)}
            />
            <Text size="sm" c="dimmed">
              {filteredLicenses.length} / {licenses.length} 件
            </Text>
            {filteredLicenses.length === 0 ? (
              <Card withBorder radius="md" p="xl">
                <Text ta="center" c="dimmed">
                  該当するライブラリはありません。
                </Text>
              </Card>
            ) : (
              <Accordion variant="separated">
                {filteredLicenses.map((item) => (
                  <Accordion.Item
                    key={`${item.ecosystem}:${item.name}@${item.version}:${item.license}`}
                    value={`${item.ecosystem}:${item.name}@${item.version}:${item.license}`}
                  >
                    <Accordion.Control>
                      <Text fw={600}>
                        {item.name}{" "}
                        <Text span c="dimmed">
                          v{item.version}
                        </Text>
                      </Text>
                      <Badge mt={4} variant="light">
                        {item.ecosystem} · {item.license}
                      </Badge>
                    </Accordion.Control>
                    <Accordion.Panel>
                      <Text
                        component="pre"
                        size="sm"
                        style={{
                          whiteSpace: "pre-wrap",
                          overflowWrap: "anywhere",
                        }}
                      >
                        {item.licenseText ||
                          `ライセンス本文をパッケージから取得できませんでした（${item.license}）。`}
                      </Text>
                    </Accordion.Panel>
                  </Accordion.Item>
                ))}
              </Accordion>
            )}
          </Stack>
        )}
      </Container>
    </Box>
  );
};

function Message({ text, color = "dimmed" }: { text: string; color?: string }) {
  return (
    <Text c={color} ta="center" py="xl">
      {text}
    </Text>
  );
}
