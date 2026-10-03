import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Accordion,
  Alert,
  Box,
  Button,
  Card,
  Checkbox,
  Code,
  Container,
  Divider,
  Group,
  Input,
  Loader,
  NumberInput,
  Paper,
  Select,
  Stack,
  Text,
  Textarea,
  TextInput,
} from "@mantine/core";
import { DateTimePicker } from "@mantine/dates";
import { CaretDownIcon, WarningIcon } from "@phosphor-icons/react";
import {
  commands,
  Index as IndexInfo,
  PluginInfo,
  PluginType,
  Indexer as IndexerInfo,
  Schedule as ScheduleInfo,
  OAuthClient,
} from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { JsonSchemaForm } from "../components/JsonSchemaForm";
import { IChangeEvent } from "@rjsf/core";
import { RJSFSchema } from "@rjsf/utils";
import { showConfirmDialog, showErrorDialog } from "../components/Dialogs";
import { RRuleTemporal, TemporalZonedDateTime } from "rrule-temporal";
import { Temporal } from "temporal-polyfill";
import { PageHeader } from "../components/PageHeader";
import { showInfoNotification } from "../components/Notifications";

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
    mutationFn: ({ indexId, name }: { indexId: number; name: string }) =>
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
      indexId: number;
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
              const onAddIndexer = async () => {
                const result = await commands.createEmptyIndexer(index.id);
                if (result.status === "error") {
                  await showErrorDialog(result.error);
                }
                queryClient.invalidateQueries({ queryKey: ["indexes"] });
                showInfoNotification("空のインデクサを追加しました");
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
                          </Box>
                        </Group>
                        <Stack gap={"md"}>
                          <IndexName
                            saving={renameIndex.isPending}
                            error={renameIndex.isError}
                            onSave={(indexId, newName) =>
                              renameIndex.mutate({ indexId, name: newName })
                            }
                            index={index}
                          />
                          <IndexDescription
                            saving={putDescription.isPending}
                            error={putDescription.isError}
                            onSave={(indexId, newDescription) =>
                              putDescription.mutate({
                                indexId,
                                description: newDescription,
                              })
                            }
                            index={index}
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
                      <Paper withBorder p="md" radius="md">
                        <Stack gap={"md"}>
                          <Group justify="space-between">
                            <Text size="xs" c="dimmed">
                              {`${"インデクサ"}：${"ファイルソースからデータを抽出します"}`}
                            </Text>
                            <Button
                              variant={"subtle"}
                              size="compact-sm"
                              onClick={onAddIndexer}
                            >
                              {"インデクサを追加"}
                            </Button>
                          </Group>
                          {index.indexers.map((v, idx) => (
                            <Indexer
                              oauthClients={listOauthClient.data ?? []}
                              key={idx}
                              num={idx + 1}
                              indexer={v}
                              fileSourceInfo={
                                plugins.find(
                                  (v) => v.pluginType === "fileSource",
                                )!
                              }
                              fileExtractorInfo={
                                plugins.find(
                                  (v) => v.pluginType === "fileExtractor",
                                )!
                              }
                            />
                          ))}
                        </Stack>
                      </Paper>
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

interface IndexNameProps {
  saving: boolean;
  error: boolean;
  index: IndexInfo;
  onSave: (indexId: number, newName: string) => void;
}
const IndexName = (props: IndexNameProps) => {
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

interface IndexDescriptionProps {
  saving: boolean;
  error: boolean;
  index: IndexInfo;
  onSave: (indexId: number, newDescription: string) => void;
}
const IndexDescription = (props: IndexDescriptionProps) => {
  const { index, onSave, saving, error } = props;
  const [isEditting, setIsEditting] = useState(false);
  const [newDesc, setNewDesc] = useState(index.description);

  const save = () => {
    const nextName = newDesc.trim();
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
        <Textarea
          label="インデックスの詳細情報"
          value={newDesc}
          onChange={(event) => setNewDesc(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Escape") onCancel();
          }}
          disabled={saving}
          error={error && "保存できませんでした。"}
          style={{ flex: 1 }}
          rows={5}
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
    <Group justify="space-between" wrap="nowrap">
      <Text size="sm" c="dimmed" style={{ whiteSpace: "pre-wrap" }}>
        {index.description.length === 0
          ? "情報がありません"
          : index.description}
      </Text>

      <Button
        variant="subtle"
        size="compact-sm"
        onClick={() => setIsEditting(true)}
        style={{ flexShrink: "0" }}
      >
        詳細情報を変更
      </Button>
    </Group>
  );
};

interface PluginSelectorProps {
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
  children?: React.ReactNode;
  onTest?: (plugin_name: string, params: any) => void;
  onAbortTest?: () => void;
  onReset?: () => void;
}

const PluginSelector = (props: PluginSelectorProps) => {
  const {
    label,
    info,
    selected,
    saving,
    error,
    plugin_type,
    onSubmit,
    initParams,
    children,
    onTest,
    onAbortTest,
    onReset,
  } = props;
  const [selecting, setSelecting] = useState(selected);
  const [paramSchema, setParamSchema] = useState<any | null>(null);
  const [resetKey, setResetKey] = useState(0);
  const [isTesting, setIsTesting] = useState(false);
  const onCancel = () => {
    setSelecting(selected);
    setResetKey((prev) => prev + 1);
    onReset?.();
  };
  const onClickTest = (plugin_name: string, params: any) => {
    if (onTest === undefined) return;
    setIsTesting(true);
    try {
      onTest(plugin_name, params);
    } finally {
      setIsTesting(false);
    }
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
            onChange={async (value, _option) => {
              if (value === null || value.length === 0) {
                return;
              }
              setSelecting(value);
            }}
            disabled={saving || isTesting}
            error={error && "保存できませんでした。"}
            style={{ flex: 1 }}
          />
          <Group gap="xs" mt={24} wrap="nowrap">
            <Button
              variant="default"
              onClick={onCancel}
              disabled={saving || isTesting}
            >
              リセット
            </Button>
          </Group>
        </Group>
        {paramSchema !== null && (
          <>
            {children}
            <Divider my="sm" />
            <JsonSchemaForm
              disabled={saving || isTesting}
              key={resetKey}
              initParams={initParams}
              jsonSchema={paramSchema}
              onSubmit={(event) => {
                if (selecting === null) return;
                onSubmit(event, selecting);
              }}
              submitButtonClosure={
                onTest === undefined
                  ? undefined
                  : (getForm) => {
                      return (_) => {
                        return (
                          <Group gap="xs" mt={24} wrap="nowrap">
                            <Button type="submit">保存</Button>
                            {isTesting ? (
                              <Button variant="outline" onClick={onAbortTest}>
                                <Loader
                                  size={"sm"}
                                  style={{ marginRight: "10px" }}
                                />
                                {onAbortTest && "中断"}
                              </Button>
                            ) : (
                              <Button
                                variant="outline"
                                onClick={() => {
                                  if (selecting === null) return;
                                  const form = getForm();
                                  if (form === null) return;
                                  const result = form.validateForm();
                                  if (result !== true) return;
                                  onClickTest(selecting, form.state.formData);
                                }}
                              >
                                テスト
                              </Button>
                            )}
                          </Group>
                        );
                      };
                    }
              }
            />
          </>
        )}
      </Stack>
    </Paper>
  );
};

interface DateInfoProps {
  label: string;
  value: string;
}

const DateInfo = (params: DateInfoProps) => {
  const { label, value } = params;
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
};

const toDtStart = (zdt: TemporalZonedDateTime) => {
  const local = zdt.toPlainDateTime();

  const value = local.toString({ smallestUnit: "second" }).replace(/[-:]/g, "");

  return `DTSTART;TZID=${zdt.timeZoneId}:${value}`;
};

interface ScheduleProps {
  indexer_id: number;
  schedule: ScheduleInfo | null;
}

const Schedule = (props: ScheduleProps) => {
  const { schedule, indexer_id } = props;
  const now = Temporal.Now.zonedDateTimeISO();
  const defaultRuleStr =
    schedule?.rfc5545Rrule ??
    `DTSTART;TZID=${toDtStart(now)}\nRRULE:FREQ=DAILY`;
  const rule = new RRuleTemporal({
    rruleString: defaultRuleStr,
  });
  const [rruleText, setRruleText] = useState(rule.toString());
  const [rrule, setRrule] = useState(rule);
  const rruleOptions = rrule.options();

  const frequency = [
    {
      label: "時間おき",
      value: "HOURLY",
    },
    {
      label: "日おき",
      value: "DAILY",
    },
    {
      label: "週おき",
      value: "WEEKLY",
    },
    {
      label: "月おき",
      value: "MONTHLY",
    },
    {
      label: "年おき",
      value: "YEARLY",
    },
  ];

  const [catchupWhenMissed, setCatchupWhenMissed] = useState(
    schedule?.catchupWhenMissed ?? false,
  );

  useEffect(() => {
    setRruleText(rrule.toString());
  }, [rrule.toString()]);

  const queryClient = useQueryClient();

  return (
    <Paper withBorder p="md" radius="md">
      <Stack gap="md">
        <Text size="xs" c="dimmed">
          {"定期実行スケジュール"}
        </Text>
        <Group align="flex-start" wrap="nowrap">
          <DateTimePicker
            label={"開始日"}
            valueFormat="YYYY-MM-DD HH:mm"
            style={{ flex: 1 }}
            value={new Date(rruleOptions.dtstart.epochMilliseconds)}
          />
          <DateTimePicker
            label={"終了日"}
            valueFormat="YYYY-MM-DD HH:mm"
            style={{ flex: 1 }}
            value={
              rruleOptions.until === undefined
                ? null
                : new Date(rruleOptions.until.epochMilliseconds)
            }
            onChange={(v) => {
              const newDt =
                v === null
                  ? undefined
                  : Temporal.PlainDateTime.from(
                      v.replace(" ", "T"),
                    ).toZonedDateTime(Temporal.Now.timeZoneId());
              const newRule = new RRuleTemporal({
                ...rruleOptions,
                until: newDt,
              });
              setRrule(newRule);
            }}
            clearable
            rightSection={<CaretDownIcon size={16} />}
            clearSectionMode="clear"
          />
        </Group>
        <Group align="flex-start" wrap="nowrap">
          <Input.Wrapper label="頻度" style={{ flex: 1 }}>
            <Group align="flex-start" gap="md" wrap="nowrap">
              <NumberInput
                style={{ width: "100px" }}
                min={1}
                allowDecimal={false}
                value={rruleOptions.interval ?? 1}
                onChange={(v) => {
                  const newRule = new RRuleTemporal({
                    ...rruleOptions,
                    interval: Number(v),
                  });
                  setRrule(newRule);
                }}
              />
              <Select
                data={frequency}
                style={{ width: "200px" }}
                value={rruleOptions.freq}
                onChange={(v) => {
                  if (v === null) return;
                  const newRule = new RRuleTemporal({
                    ...rruleOptions,
                    freq: v as any,
                  });
                  setRrule(newRule);
                }}
              />
            </Group>
          </Input.Wrapper>
        </Group>
        <Checkbox
          label="非起動時に予定を過ぎた場合、次回起動時に処理を開始する"
          checked={catchupWhenMissed}
          onChange={(e) => {
            setCatchupWhenMissed(e.currentTarget.checked);
          }}
        />
        <Divider />
        <Group align="flex-start" wrap="nowrap">
          <TextInput
            label={"RFC5545 RRULE 直接入力"}
            value={rruleText}
            onChange={(e) => {
              const v = e.currentTarget.value;
              setRruleText(v);
              try {
                const newRule = new RRuleTemporal({
                  rruleString: v,
                });
                setRrule(newRule);
              } catch {}
            }}
            style={{ flex: 1 }}
          />
          <Group gap="xs" mt={24} wrap="nowrap">
            <Button
              variant="default"
              onClick={() => {
                const newRule = new RRuleTemporal({
                  rruleString: defaultRuleStr,
                });
                setRrule(newRule);
              }}
            >
              リセット
            </Button>
          </Group>
        </Group>
        <Input.Wrapper label={"実行予定日時プレビュー"}>
          <Code style={{ display: "block", whiteSpace: "pre-wrap" }}>
            {rrule
              .all((_, i) => i < 6)
              .map((v) => v.toLocaleString())
              .join("\n")}
          </Code>
        </Input.Wrapper>
        <Group gap="xs" mt={24} wrap="nowrap">
          <Button
            size="sm"
            onClick={async () => {
              const newSchedule: ScheduleInfo = {
                id: 0,
                indexerId: indexer_id,
                rfc5545Rrule: rrule.toString(),
                catchupWhenMissed: catchupWhenMissed,
              };
              const result = await commands.upsertSchedule(
                indexer_id,
                newSchedule,
              );
              if (result.status === "error") {
                await showErrorDialog(result.error);
              }
              queryClient.invalidateQueries({ queryKey: ["indexes"] });
              showInfoNotification("スケジュールを保存しました");
            }}
          >
            {"保存"}
          </Button>
          {schedule !== null && (
            <Button
              variant="outline"
              size="sm"
              onClick={async () => {
                await showConfirmDialog(
                  "定期実行スケジュールを削除します、よろしいですか？",
                  async (ok) => {
                    if (ok) {
                      const result = await commands.deleteSchedule(indexer_id);
                      if (result.status === "error") {
                        await showErrorDialog(result.error);
                      }
                      queryClient.invalidateQueries({ queryKey: ["indexes"] });
                      showInfoNotification("スケジュールを削除しました");
                    }
                  },
                );
              }}
            >
              {"削除"}
            </Button>
          )}
          {schedule === null && (
            <Alert
              color="yellow"
              title="スケジュールは未設定です"
              icon={<WarningIcon />}
            />
          )}
        </Group>
      </Stack>
    </Paper>
  );
};

interface IndexerProps {
  indexer: IndexerInfo;
  fileSourceInfo: PluginInfo;
  fileExtractorInfo: PluginInfo;
  oauthClients: OAuthClient[];
  num: number;
}

const Indexer = (params: IndexerProps) => {
  const { indexer, fileSourceInfo, fileExtractorInfo, num, oauthClients } =
    params;
  const [oauthId, setOauthId] = useState(
    indexer.fileSource?.oauthClient?.id ?? null,
  );
  const queryClient = useQueryClient();
  return (
    <Paper withBorder p="md" radius="md">
      <Stack gap={"sm"}>
        <Text size="xs" c="dimmed">
          {`インデクサ ${num}`}
        </Text>
        <Schedule schedule={indexer.schedule} indexer_id={indexer.id} />
        <PluginSelector
          label="ファイルソース：ファイルの取得元です"
          saving={false}
          error={false}
          info={fileSourceInfo}
          selected={indexer.fileSource?.name ?? null}
          plugin_type="fileSource"
          onSubmit={async (event, name) => {
            const result = await commands.upsertFileSource(
              indexer.id,
              name,
              JSON.stringify(event.formData),
              oauthId,
            );
            if (result.status === "error") {
              await showErrorDialog(result.error);
            }
            queryClient.invalidateQueries({ queryKey: ["indexes"] });
            showInfoNotification("ファイルソースを保存しました");
          }}
          initParams={indexer.fileSource?.params}
          onReset={() => {
            setOauthId(indexer.fileSource?.oauthClient?.id ?? null);
          }}
          onTest={async (name, params) => {
            const result = await commands.testFileSource(
              name,
              JSON.stringify(params),
              oauthId,
            );
            if (result.status === "error") {
              await showErrorDialog(result.error);
            } else {
              showInfoNotification("ファイルソースのテストに成功しました");
            }
          }}
          onAbortTest={
            oauthId === null
              ? undefined
              : () => {
                  const oauthClient = oauthClients.find(
                    (v) => v.id === oauthId,
                  )?.clientId;
                  if (oauthClient === undefined) return;
                  commands.abortOauth(oauthClient);
                }
          }
        >
          <Select
            label={"OAuthクライアント"}
            value={oauthId}
            data={oauthClients.map((v) => ({ label: v.name, value: v.id }))}
            clearable
            onChange={(v) => {
              setOauthId(v);
            }}
          />
        </PluginSelector>
        <PluginSelector
          label="ファイル抽出：ファイルを抽出します"
          saving={false}
          error={false}
          info={fileExtractorInfo}
          selected={indexer.fileExtractor?.name ?? null}
          plugin_type="fileExtractor"
          onSubmit={async (event, name) => {
            const result = await commands.upsertFileExtractor(
              indexer.id,
              name,
              JSON.stringify(event.formData),
            );
            if (result.status === "error") {
              await showErrorDialog(result.error);
            }
            queryClient.invalidateQueries({ queryKey: ["indexes"] });
            showInfoNotification("ファイル抽出を変更しました");
          }}
          initParams={indexer.fileExtractor?.params}
        />
        <Group gap="xs" mt={24} wrap="nowrap">
          <Button size="sm">{"今すぐ実行"}</Button>
          <Button
            variant="outline"
            size="sm"
            onClick={async () => {
              await showConfirmDialog(
                "インデクサを削除します、よろしいですか？",
                async (ok) => {
                  if (ok) {
                    const result = await commands.deleteIndexer(indexer.id);
                    if (result.status === "error") {
                      await showErrorDialog(result.error);
                    }
                    queryClient.invalidateQueries({ queryKey: ["indexes"] });
                    showInfoNotification("インデクサを削除しました");
                  }
                },
              );
            }}
          >
            {"インデクサを削除"}
          </Button>
        </Group>
      </Stack>
    </Paper>
  );
};
