import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Accordion,
  Box,
  Button,
  Card,
  Container,
  Group,
  Paper,
  Stack,
  Text,
} from "@mantine/core";
import { commands } from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { IChangeEvent } from "@rjsf/core";
import { RJSFSchema } from "@rjsf/utils";
import { showConfirmDialog, showErrorDialog } from "../components/Dialogs";
import { PageHeader } from "../components/PageHeader";
import { showInfoNotification } from "../components/Notifications";
import {
  FileSourceSelector,
  PluginSelector,
} from "../components/PluginSelector";
import { ScheduleInput } from "../components/Schedule";
import { DateInfo } from "../components/DateInfo";
import {
  EditableTextArea,
  EditableTextInput,
} from "../components/EditableTextInput";

type SortKey = "name" | "lastIndexingStartedAt";
type Direction = "asc" | "desc";

export const Index = () => {
  const queryClient = useQueryClient();
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [direction, setDirection] = useState<Direction>("desc");
  const [open, setOpen] = useState<string[]>([]);
  const {
    data: indexes = [],
    isLoading,
    isError,
  } = useQuery({
    queryKey: ["indexes"],
    queryFn: async () => {
      const result = await commands.listIndexes();
      if (result.status === "error") {
        await showErrorDialog(result.error);
        throw new Error(result.error);
      }
      return result.data;
    },
  });
  const listOauthClient = useQuery({
    queryKey: ["oauths"],
    queryFn: async () => {
      const result = await commands.listOauthClient();
      if (result.status === "error") {
        await showErrorDialog(result.error);
        throw new Error(result.error);
      }
      return result.data;
    },
  });
  const { data: plugins = [] } = useQuery({
    queryKey: ["plugins"],
    queryFn: async () => await unwrapCommandResult(commands.getPluginInfo()),
  });
  const createIndex = useMutation({
    mutationFn: () => unwrapCommandResult(commands.createEmptyIndex()),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["indexes"] });
      showInfoNotification("空のインデクサを追加しました");
    },
  });
  const renameIndex = useMutation({
    mutationFn: ({ indexId, name }: { indexId: string; name: string }) =>
      unwrapCommandResult(commands.renameIndex(indexId, name)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["indexes"] });
      showInfoNotification("インデックス名を変更しました");
    },
  });
  const putDescription = useMutation({
    mutationFn: ({
      indexId,
      description,
    }: {
      indexId: string;
      description: string;
    }) => unwrapCommandResult(commands.putDescription(indexId, description)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["indexes"] });
      showInfoNotification("インデックスの説明を変更しました");
    },
  });
  const sortedIndexes = useMemo(
    () =>
      [...indexes].sort((a, b) => {
        if (sortKey === "name") {
          const result = a.index.name.localeCompare(b.index.name, "ja");
          return direction === "asc" ? result : -result;
        }

        const aStartedAt = a.lastIndexingStartedAt
          ? Date.parse(a.lastIndexingStartedAt)
          : null;
        const bStartedAt = b.lastIndexingStartedAt
          ? Date.parse(b.lastIndexingStartedAt)
          : null;
        if (aStartedAt === null) return bStartedAt === null ? 0 : 1;
        if (bStartedAt === null) return -1;

        const result = aStartedAt - bStartedAt;
        return direction === "asc" ? result : -result;
      }),
    [sortKey, direction, indexes],
  );
  const chooseSort = (key: SortKey) => {
    if (sortKey === key)
      setDirection((value) => (value === "asc" ? "desc" : "asc"));
    else {
      setSortKey(key);
      setDirection(key === "name" ? "asc" : "desc");
    }
  };
  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <Group justify="space-between" align="flex-end" mb={28}>
          <PageHeader
            pageName="インデックス"
            description="ファイルソースと検索設定を管理します"
          />
          <Button
            size="md"
            onClick={() => createIndex.mutate()}
            loading={createIndex.isPending}
          >
            ＋ 新しいインデックス
          </Button>
        </Group>
        <Card withBorder radius="md" padding="sm" mb="md">
          <Group justify="space-between">
            <Group gap={4}>
              <Text size="sm" fw={600} c="dimmed" mr="xs">
                並び替え
              </Text>
              {(
                [
                  ["name", "名前"],
                  ["lastIndexingStartedAt", "最終実行日時"],
                ] as [SortKey, string][]
              ).map(([key, label]) => (
                <Button
                  key={key}
                  variant={sortKey === key ? "light" : "subtle"}
                  color="gray"
                  size="compact-sm"
                  onClick={() => chooseSort(key)}
                >
                  {label}
                  {sortKey === key ? (direction === "asc" ? " ↑" : " ↓") : ""}
                </Button>
              ))}
            </Group>
            <Text size="sm" c="dimmed">
              {indexes.length} 件のインデックス
            </Text>
          </Group>
        </Card>
        {isLoading && (
          <Text c="dimmed" ta="center" py="xl">
            インデックスを読み込んでいます…
          </Text>
        )}
        {isError && (
          <Text c="red" ta="center" py="xl">
            インデックスを取得できませんでした。
          </Text>
        )}
        {!isLoading && !isError && (
          <Accordion
            multiple
            value={open}
            onChange={setOpen}
            variant="separated"
            radius="md"
          >
            {sortedIndexes.map(({ index, lastIndexingStartedAt }) => {
              const onSubmitEmbedder = async (
                event: IChangeEvent<any, RJSFSchema, any>,
                plugin_name: string,
              ) => {
                const result = await commands.upsertEmbedder(
                  index.id,
                  plugin_name,
                  JSON.stringify(event.formData),
                );
                if (result.status === "error") {
                  await showErrorDialog(result.error);
                }
                queryClient.invalidateQueries({ queryKey: ["indexes"] });
                showInfoNotification("ベクトル化プラグインを変更しました");
              };
              const onSubmitVectorStore = async (
                event: IChangeEvent<any, RJSFSchema, any>,
                plugin_name: string,
              ) => {
                const result = await commands.upsertVectorStore(
                  index.id,
                  plugin_name,
                  JSON.stringify(event.formData),
                );
                if (result.status === "error") {
                  await showErrorDialog(result.error);
                }
                queryClient.invalidateQueries({ queryKey: ["indexes"] });
                showInfoNotification("ベクトルストアを変更しました");
              };
              return (
                <Accordion.Item value={index.id.toString()} key={index.id}>
                  <Group gap={0} wrap="nowrap" align="stretch">
                    <Accordion.Control style={{ flex: 1 }}>
                      <Group justify="space-between" wrap="nowrap" pr="md">
                        <Box>
                          <Group gap="sm">
                            <Text fw={700} size="lg">
                              {index.name}
                            </Text>
                          </Group>
                          <Text
                            size="sm"
                            c="dimmed"
                            mt={3}
                            style={{ whiteSpace: "pre-wrap" }}
                          >
                            {index.description || ""}
                          </Text>
                        </Box>
                        <Group gap="xl" visibleFrom="sm">
                          <DateInfo
                            label="最終実行日時"
                            value={
                              lastIndexingStartedAt
                                ? new Date(lastIndexingStartedAt)
                                : null
                            }
                          />
                        </Group>
                      </Group>
                    </Accordion.Control>
                  </Group>
                  <Accordion.Panel>
                    <Stack gap="lg" pb="sm">
                      <Paper withBorder p="md" radius="md">
                        <Group justify="space-between" mb="md">
                          <Box>
                            <Text fw={700}>基本情報</Text>
                          </Box>
                        </Group>
                        <Stack gap={"md"}>
                          <EditableTextInput
                            label="インデックス名"
                            value={index.name}
                            saving={renameIndex.isPending}
                            error={renameIndex.isError}
                            onSave={(newName) =>
                              renameIndex.mutate({
                                indexId: index.id,
                                name: newName,
                              })
                            }
                          />
                          <EditableTextArea
                            saving={putDescription.isPending}
                            error={putDescription.isError}
                            value={index.description}
                            emptyValue="情報がありません"
                            label="インデックスの詳細情報"
                            onSave={(newDescription) =>
                              putDescription.mutate({
                                indexId: index.id,
                                description: newDescription,
                              })
                            }
                            row={5}
                          />
                          <Group gap="xs" mt={24} wrap="nowrap">
                            <Button
                              size="sm"
                              color="red"
                              onClick={async () => {
                                await showConfirmDialog(
                                  "インデックスを削除します、よろしいですか？",
                                  async (ok) => {
                                    if (ok) {
                                      const result = await commands.deleteIndex(
                                        index.id,
                                      );
                                      if (result.status === "error") {
                                        await showErrorDialog(result.error);
                                      }
                                      queryClient.invalidateQueries({
                                        queryKey: ["indexes"],
                                      });
                                    }
                                  },
                                );
                              }}
                            >
                              {"インデックスを削除"}
                            </Button>
                          </Group>
                        </Stack>
                      </Paper>
                      <ScheduleInput
                        schedule={index.schedule}
                        index_id={index.id}
                        onUpsertSchedule={async (index_id, newSchedule) => {
                          const result = await commands.upsertSchedule(
                            index_id,
                            newSchedule,
                          );
                          if (result.status === "error") {
                            await showErrorDialog(result.error);
                          }
                          queryClient.invalidateQueries({
                            queryKey: ["indexes"],
                          });
                          showInfoNotification("スケジュールを保存しました");
                        }}
                        onDeleteSchedule={async (index_id) => {
                          await showConfirmDialog(
                            "定期実行スケジュールを削除します、よろしいですか？",
                            async (ok) => {
                              if (ok) {
                                const result = await commands.upsertSchedule(
                                  index_id,
                                  null,
                                );
                                if (result.status === "error") {
                                  await showErrorDialog(result.error);
                                }
                                queryClient.invalidateQueries({
                                  queryKey: ["indexes"],
                                });
                                showInfoNotification(
                                  "スケジュールを削除しました",
                                );
                              }
                            },
                          );
                        }}
                      />
                      <FileSourceSelector
                        label="ファイルソース：ファイルの取得元です"
                        saving={false}
                        error={false}
                        info={
                          plugins.find((v) => v.pluginType === "fileSource")!
                        }
                        index={index}
                        plugin_type="fileSource"
                        onSubmit={async (event, name, oauthClient) => {
                          const result = await commands.upsertFileSource(
                            index.id,
                            name,
                            JSON.stringify(event.formData),
                            oauthClient?.id ?? null,
                          );
                          if (result.status === "error") {
                            await showErrorDialog(result.error);
                          }
                          queryClient.invalidateQueries({
                            queryKey: ["indexes"],
                          });
                          showInfoNotification("ファイルソースを保存しました");
                        }}
                        initParams={index.fileSource?.params}
                        oauthClients={listOauthClient.data ?? []}
                      />
                      <PluginSelector
                        label="ファイル抽出：ファイルを抽出します"
                        saving={false}
                        error={false}
                        info={
                          plugins.find((v) => v.pluginType === "fileExtractor")!
                        }
                        selected={index.fileExtractor?.name ?? null}
                        plugin_type="fileExtractor"
                        onSubmit={async (event, name) => {
                          const result = await commands.upsertFileExtractor(
                            index.id,
                            name,
                            JSON.stringify(event.formData),
                          );
                          if (result.status === "error") {
                            await showErrorDialog(result.error);
                          }
                          queryClient.invalidateQueries({
                            queryKey: ["indexes"],
                          });
                          showInfoNotification("ファイル抽出を変更しました");
                        }}
                        initParams={index.fileExtractor?.params}
                      />
                      <PluginSelector
                        label="ベクトル化：ファイルを検索可能に変換します"
                        saving={false}
                        error={false}
                        info={plugins.find((v) => v.pluginType === "embedder")!}
                        selected={index.embedder?.name ?? null}
                        plugin_type="embedder"
                        onSubmit={onSubmitEmbedder}
                        initParams={index.embedder?.params}
                      />
                      <PluginSelector
                        label="ベクトルストア：生成したベクトルを保存し、検索に使います"
                        saving={false}
                        error={false}
                        info={
                          plugins.find((v) => v.pluginType === "vectorStore")!
                        }
                        selected={index.vectorStore?.name ?? null}
                        plugin_type="vectorStore"
                        onSubmit={onSubmitVectorStore}
                        initParams={index.vectorStore?.params}
                      />
                      <Group gap="xs" mt={24} wrap="nowrap">
                        <Button
                          size="sm"
                          onClick={async () => {
                            const result = await commands.startIndexing(
                              index.id,
                            );
                            if (result.status === "error") {
                              await showErrorDialog(result.error);
                            }
                          }}
                        >
                          {"今すぐ実行"}
                        </Button>
                      </Group>
                    </Stack>
                  </Accordion.Panel>
                </Accordion.Item>
              );
            })}
          </Accordion>
        )}
      </Container>
    </Box>
  );
};
