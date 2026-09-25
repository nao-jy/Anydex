import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Accordion,
  Box,
  Button,
  Card,
  Container,
  Divider,
  Group,
  Paper,
  Select,
  Stack,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import {
  commands,
  Index as IndexInfo,
  PluginInfo,
  PluginType,
} from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { JsonSchemaForm } from "../components/JsonSchemaForm";
import { IChangeEvent } from "@rjsf/core";
import { RJSFSchema } from "@rjsf/utils";
import { showErrorDialog } from "../components/Dialogs";

type SortKey = "name" | "createdAt" | "updatedAt";
type Direction = "asc" | "desc";

export const Index = () => {
  const queryClient = useQueryClient();
  const [sortKey, setSortKey] = useState<SortKey>("updatedAt");
  const [direction, setDirection] = useState<Direction>("desc");
  const [open, setOpen] = useState<string[]>([]);
  const {
    data: indexes = [],
    isLoading,
    isError,
  } = useQuery({
    queryKey: ["indexes"],
    queryFn: () => unwrapCommandResult(commands.listIndexes()),
  });
  const { data: plugins = [] } = useQuery({
    queryKey: ["plugins"],
    queryFn: async () => await unwrapCommandResult(commands.getPluginInfo()),
  });
  const createIndex = useMutation({
    mutationFn: () => unwrapCommandResult(commands.createEmptyIndex()),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["indexes"] }),
  });
  const renameIndex = useMutation({
    mutationFn: ({ indexId, name }: { indexId: number; name: string }) =>
      unwrapCommandResult(commands.renameIndex(indexId, name)),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["indexes"] });
    },
  });
  const sortedIndexes = useMemo(
    () =>
      [...indexes].sort((a, b) => {
        const result = a[sortKey].localeCompare(b[sortKey], "ja");
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
          <Box>
            <Text size="sm" c="indigo" fw={700} tt="uppercase" lts={1}>
              Search indexes
            </Text>
            <Title order={1} mt={4}>
              インデックス
            </Title>
            <Text c="dimmed" mt={6}>
              ファイルソースと検索設定を管理します
            </Text>
          </Box>
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
                  ["createdAt", "作成日"],
                  ["updatedAt", "最終更新日"],
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
            {sortedIndexes.map((index) => {
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
              };
              console.log(index);
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
                          <Text size="sm" c="dimmed" mt={3}>
                            {index.description ||
                              "説明はまだ設定されていません"}
                          </Text>
                        </Box>
                        <Group gap="xl" visibleFrom="sm">
                          <DateInfo label="作成日" value={index.createdAt} />
                          <DateInfo
                            label="最終更新日"
                            value={index.updatedAt}
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
                            <Text size="sm" c="dimmed">
                              このインデックスを識別する名前です
                            </Text>
                          </Box>
                        </Group>
                        <IndexName
                          saving={renameIndex.isPending}
                          error={renameIndex.isError}
                          onSave={(indexId, newName) =>
                            renameIndex.mutate({ indexId, name: newName })
                          }
                          index={index}
                        />
                      </Paper>
                      <Setting
                        label="インデクサ"
                        value={`${index.indexers.length} 件`}
                        detail="ファイルソースからデータを抽出します"
                        setting_label="インデクサを追加"
                        on_click_setting={() => {}}
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
                          plugins.find((v) => v.pluginType === "vector-store")!
                        }
                        selected={index.vectorStore?.name ?? null}
                        plugin_type="vectorStore"
                        onSubmit={onSubmitVectorStore}
                        initParams={index.vectorStore?.params}
                      />
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

const IndexName = (props: {
  saving: boolean;
  error: boolean;
  index: IndexInfo;
  onSave: (indexId: number, newName: string) => void;
}) => {
  const { index, onSave, saving, error } = props;
  const [isEditting, setIsEditting] = useState(false);
  const [newName, setNewName] = useState(index.name);

  const save = () => {
    const nextName = newName.trim();
    if (!nextName) {
      return;
    }
    onSave(index.id, nextName);
    setIsEditting(false);
  };

  const onCancel = () => {
    setIsEditting(false);
  };

  if (isEditting) {
    return (
      <Group align="flex-start" wrap="nowrap">
        <TextInput
          label="インデックス名"
          value={newName}
          onChange={(event) => setNewName(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") save();
            if (event.key === "Escape") onCancel();
          }}
          disabled={saving}
          error={error && "保存できませんでした。"}
          style={{ flex: 1 }}
        />
        <Group gap="xs" mt={24} wrap="nowrap">
          <Button variant="default" onClick={onCancel} disabled={saving}>
            キャンセル
          </Button>
          <Button onClick={save} loading={saving}>
            保存
          </Button>
        </Group>
      </Group>
    );
  }

  return (
    <Group justify="space-between">
      <Text fw={600}>{index.name}</Text>
      <Button
        variant="subtle"
        size="compact-sm"
        onClick={() => setIsEditting(true)}
      >
        名前を変更
      </Button>
    </Group>
  );
};

function PluginSelector(props: {
  label: string;
  saving: boolean;
  error: boolean;
  info: PluginInfo;
  selected: string | null;
  plugin_type: PluginType;
  onSubmit: (
    event: IChangeEvent<any, RJSFSchema, any>,
    plugin_name: string,
  ) => void;
  initParams: any | null;
}) {
  const {
    label,
    info,
    selected,
    saving,
    error,
    plugin_type,
    onSubmit,
    initParams,
  } = props;
  const [selecting, setSelecting] = useState(selected);
  const [paramSchema, setParamSchema] = useState<any | null>(null);
  const onCancel = () => {
    setSelecting(selected);
  };
  useEffect(() => {
    const asyncSchema = async () => {
      if (selecting === null || selecting.length === 0) {
        setParamSchema(null);
        return;
      }
      const paramSchema = await commands.getParamSchema(plugin_type, selecting);
      if (paramSchema === null) {
        setParamSchema(null);
      } else {
        try {
          const parsed = JSON.parse(paramSchema);
          setParamSchema(parsed);
        } catch {
          setParamSchema(null);
        }
      }
    };
    asyncSchema();
  }, [selecting]);

  return (
    <Paper withBorder p="md" radius="md">
      <Stack gap="md">
        <Text size="xs" c="dimmed">
          {label}
        </Text>
        <Group align="flex-start" wrap="nowrap">
          <Select
            label={info.pluginType}
            value={selecting}
            data={info.plugins}
            onChange={async (_value, option) => {
              if (option === null || option.value.length === 0) {
                setParamSchema(null);
                return;
              }
              setSelecting(option.value);
            }}
            disabled={saving}
            error={error && "保存できませんでした。"}
            style={{ flex: 1 }}
          />
          <Group gap="xs" mt={24} wrap="nowrap">
            <Button variant="default" onClick={onCancel} disabled={saving}>
              リセット
            </Button>
          </Group>
        </Group>
        {paramSchema !== null && (
          <>
            <Divider my="sm" />
            <JsonSchemaForm
              initParams={initParams}
              jsonSchema={paramSchema}
              onSubmit={(event) => {
                if (selecting === null) return;
                onSubmit(event, selecting);
              }}
            />
          </>
        )}
      </Stack>
    </Paper>
  );
}

function DateInfo({ label, value }: { label: string; value: string }) {
  const date = new Date(value);
  const localValue = Number.isNaN(date.getTime())
    ? value
    : date.toLocaleString();

  return (
    <Box>
      <Text size="xs" c="dimmed">
        {label}
      </Text>
      <Text size="sm">{localValue}</Text>
    </Box>
  );
}
function Setting({
  label,
  detail,
  setting_label,
  on_click_setting,
}: {
  label: string;
  value: string;
  detail: string;
  setting_label: string;
  on_click_setting: () => void;
}) {
  return (
    <Paper withBorder p="md" radius="md">
      <Text size="xs" c="dimmed">
        {`${label}：${detail}`}
      </Text>
      <Group justify="space-between">
        <Group gap="xs" mt={3}>
          未設定
        </Group>
        <Button variant={"subtle"} size="compact-sm" onClick={on_click_setting}>
          {setting_label}
        </Button>
      </Group>
    </Paper>
  );
}

const weekdays: Record<string, string> = {
  MO: "月",
  TU: "火",
  WE: "水",
  TH: "木",
  FR: "金",
  SA: "土",
  SU: "日",
};

function formatSchedule(rule: string) {
  const fields = Object.fromEntries(
    rule
      .split(";")
      .map((field) => field.split("=", 2))
      .filter((field): field is [string, string] => field.length === 2),
  );
  const interval =
    fields.INTERVAL && fields.INTERVAL !== "1" ? Number(fields.INTERVAL) : 1;
  if (fields.FREQ === "DAILY")
    return interval === 1 ? "毎日" : `${interval} 日おき`;
  if (fields.FREQ === "WEEKLY") {
    const days = fields.BYDAY?.split(",")
      .map((day) => weekdays[day] ?? day)
      .join("・");
    return `${interval === 1 ? "毎週" : `${interval} 週おき`}${days ? ` ${days}曜` : ""}`;
  }
  if (fields.FREQ === "MONTHLY")
    return interval === 1 ? "毎月" : `${interval} か月おき`;
  return rule;
}

function formatDateTime(value: string, timezone: string) {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) return value;
  return new Intl.DateTimeFormat("ja-JP", {
    dateStyle: "short",
    timeStyle: "short",
    timeZone: timezone,
  }).format(date);
}
