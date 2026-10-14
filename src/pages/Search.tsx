import { useMemo, useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import {
  ActionIcon,
  Box,
  Breadcrumbs,
  Button,
  Card,
  Container,
  Divider,
  Group,
  Loader,
  Modal,
  ScrollArea,
  Select,
  Stack,
  Text,
  ThemeIcon,
  Title,
  Tooltip,
  UnstyledButton,
} from "@mantine/core";
import {
  IconChevronDown,
  IconChevronLeft,
  IconChevronRight,
  IconChevronUp,
  IconDownload,
  IconFile,
  IconFolder,
  IconFolderOpen,
} from "@tabler/icons-react";
import { save } from "@tauri-apps/plugin-dialog";
import { commands, type DirectoryEntry, type FileMetadata } from "../bindings";
import { showErrorDialog } from "../components/Dialogs";
import { showInfoNotification } from "../components/Notifications";
import { PageHeader } from "../components/PageHeader";
import { formatDateTime } from "../components/utils";
import { unwrapCommandResult } from "../tauri";
import styles from "./Search.module.css";

type SortColumn = "name" | "updatedAt" | "size" | "kind";
type SortDirection = "asc" | "desc";

export const Search = () => {
  const [selectedIndexId, setSelectedIndexId] = useState<string | null>(null);
  const [directoryPath, setDirectoryPath] = useState("/");
  const [selectedFileUri, setSelectedFileUri] = useState<string | null>(null);
  const [sortColumn, setSortColumn] = useState<SortColumn>("name");
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");

  const indexesQuery = useQuery({
    queryKey: ["indexes"],
    queryFn: () => unwrapCommandResult(commands.listIndexes()),
  });
  const indexes = indexesQuery.data ?? [];
  const indexId =
    selectedIndexId && indexes.some(({ index }) => index.id === selectedIndexId)
      ? selectedIndexId
      : (indexes[0]?.index.id ?? null);
  const currentIndex = indexes.find(({ index }) => index.id === indexId)?.index;

  const directoryQuery = useQuery({
    queryKey: ["directory", indexId, directoryPath],
    queryFn: () =>
      unwrapCommandResult(commands.listDirectory(indexId!, directoryPath)),
    enabled: indexId !== null,
  });
  const fileInfoQuery = useQuery({
    queryKey: ["fileInfo", indexId, selectedFileUri],
    queryFn: () =>
      unwrapCommandResult(commands.getFileInfo(indexId!, selectedFileUri!)),
    enabled: indexId !== null && selectedFileUri !== null,
  });
  const downloadMutation = useMutation({
    mutationFn: async (file: FileMetadata) => {
      const destination = await save({ defaultPath: file.name });
      if (!destination) return false;
      await unwrapCommandResult(
        commands.downloadFile(indexId!, file.fileUri, destination),
      );
      return true;
    },
    onSuccess: (saved) => {
      if (saved) showInfoNotification("ファイルをダウンロードしました");
    },
    onError: (error) => {
      void showErrorDialog(
        `ファイルをダウンロードできませんでした: ${getErrorMessage(error)}`,
      );
    },
  });

  const entries = useMemo(
    () =>
      sortEntries(
        directoryQuery.data?.entries ?? [],
        sortColumn,
        sortDirection,
      ),
    [directoryQuery.data, sortColumn, sortDirection],
  );
  const selectedFile = fileInfoQuery.data?.file;
  const pathSegments = directoryPath.split("/").filter(Boolean);

  const openEntry = (entry: DirectoryEntry) => {
    if (entry.isDirectory) {
      setDirectoryPath(entry.path);
      setSelectedFileUri(null);
    } else if (entry.fileUri) {
      setSelectedFileUri(entry.fileUri);
    }
  };
  const navigateTo = (path: string) => {
    setDirectoryPath(path);
    setSelectedFileUri(null);
  };
  const chooseSort = (column: SortColumn) => {
    if (column === sortColumn) {
      setSortDirection((direction) => (direction === "asc" ? "desc" : "asc"));
      return;
    }
    setSortColumn(column);
    setSortDirection("asc");
  };
  const parentPath =
    directoryPath === "/"
      ? "/"
      : directoryPath.slice(0, directoryPath.lastIndexOf("/")) || "/";

  return (
    <Box bg="var(--mantine-color-body)" py={40}>
      <Container size="xl">
        <PageHeader
          pageName="検索"
          description="インデックス内のファイルを閲覧し、メタデータを確認します"
        />

        <Card withBorder radius="md" padding="md" mb="md">
          <Group justify="space-between" align="flex-end">
            <Select
              label="インデックス"
              placeholder="インデックスを選択"
              data={indexes.map(({ index }) => ({
                value: index.id,
                label: index.name,
              }))}
              value={indexId}
              onChange={(value) => {
                setSelectedIndexId(value);
                navigateTo("/");
              }}
              disabled={indexesQuery.isLoading || indexes.length === 0}
              w={{ base: "100%", sm: 320 }}
            />
            {currentIndex && (
              <Text size="sm" c="dimmed">
                {currentIndex.description || "説明はありません"}
              </Text>
            )}
          </Group>
        </Card>

        {indexesQuery.isLoading && (
          <StatusMessage text="インデックスを読み込んでいます…" loading />
        )}
        {indexesQuery.isError && (
          <StatusMessage
            text={`インデックスを取得できませんでした: ${getErrorMessage(indexesQuery.error)}`}
            color="red"
          />
        )}
        {!indexesQuery.isLoading &&
          !indexesQuery.isError &&
          indexes.length === 0 && (
            <StatusMessage text="インデックスがありません。先にインデックスを作成してください。" />
          )}

        {indexId && (
          <>
            <Card withBorder radius="md" padding="sm" mb="md">
              <Group justify="space-between" wrap="nowrap">
                <Group gap="xs" wrap="nowrap" style={{ overflowX: "auto" }}>
                  <Tooltip label="上の階層へ">
                    <ActionIcon
                      variant="subtle"
                      aria-label="上の階層へ"
                      onClick={() => navigateTo(parentPath)}
                      disabled={directoryPath === "/"}
                    >
                      <IconChevronLeft size={18} />
                    </ActionIcon>
                  </Tooltip>
                  <Breadcrumbs
                    separator={<IconChevronRight size={14} />}
                    styles={{ breadcrumb: { whiteSpace: "nowrap" } }}
                  >
                    <Button
                      variant={pathSegments.length === 0 ? "light" : "subtle"}
                      size="compact-sm"
                      onClick={() => navigateTo("/")}
                    >
                      root
                    </Button>
                    {pathSegments.map((segment, index) => {
                      const path = `/${pathSegments.slice(0, index + 1).join("/")}`;
                      return (
                        <Button
                          key={path}
                          variant={
                            index === pathSegments.length - 1
                              ? "light"
                              : "subtle"
                          }
                          size="compact-sm"
                          onClick={() => navigateTo(path)}
                        >
                          {segment}
                        </Button>
                      );
                    })}
                  </Breadcrumbs>
                </Group>
                <Text size="sm" c="dimmed" style={{ flexShrink: 0 }}>
                  {entries.length} 件
                </Text>
              </Group>
            </Card>

            <Card withBorder radius="md" padding={0} style={{ minWidth: 0 }}>
              {directoryQuery.isLoading ? (
                <StatusMessage text="ファイル一覧を読み込んでいます…" loading />
              ) : directoryQuery.isError ? (
                <StatusMessage
                  text={`ファイル一覧を取得できませんでした: ${getErrorMessage(directoryQuery.error)}`}
                  color="red"
                />
              ) : (
                <>
                  <Box className={styles.listHeader}>
                    <SortHeader
                      label="名前"
                      column="name"
                      activeColumn={sortColumn}
                      direction={sortDirection}
                      onClick={chooseSort}
                    />
                    <SortHeader
                      label="最終更新日"
                      column="updatedAt"
                      activeColumn={sortColumn}
                      direction={sortDirection}
                      onClick={chooseSort}
                      className={styles.dateColumn}
                    />
                    <SortHeader
                      label="サイズ"
                      column="size"
                      activeColumn={sortColumn}
                      direction={sortDirection}
                      onClick={chooseSort}
                      className={styles.sizeColumn}
                      align="right"
                    />
                    <SortHeader
                      label="種類"
                      column="kind"
                      activeColumn={sortColumn}
                      direction={sortDirection}
                      onClick={chooseSort}
                      align="right"
                    />
                    <span aria-hidden="true" />
                  </Box>
                  <Divider />
                  {entries.length === 0 ? (
                    <Stack align="center" justify="center" gap="xs" py={48}>
                      <ThemeIcon
                        variant="light"
                        color="gray"
                        size={42}
                        radius="xl"
                      >
                        <IconFolderOpen size={22} />
                      </ThemeIcon>
                      <Text size="sm" c="dimmed">
                        このフォルダは空です
                      </Text>
                    </Stack>
                  ) : (
                    <ScrollArea.Autosize mah="60vh" type="auto">
                      <Stack gap={0} p={6}>
                        {entries.map((entry) => (
                          <FileEntry
                            key={entry.path}
                            entry={entry}
                            selected={entry.fileUri === selectedFileUri}
                            onClick={() => openEntry(entry)}
                          />
                        ))}
                      </Stack>
                    </ScrollArea.Autosize>
                  )}
                </>
              )}
            </Card>

            <Modal
              opened={selectedFileUri !== null}
              onClose={() => setSelectedFileUri(null)}
              centered
              size="lg"
              title={
                <Group gap="sm">
                  <ThemeIcon
                    variant="light"
                    color="indigo"
                    size="lg"
                    radius="md"
                  >
                    <IconFile size={20} />
                  </ThemeIcon>
                  <Title order={3}>ファイル情報</Title>
                </Group>
              }
              overlayProps={{ backgroundOpacity: 0.35, blur: 2 }}
              transitionProps={{ transition: "fade", duration: 200 }}
            >
              <FileDetails
                file={selectedFile}
                isLoading={fileInfoQuery.isLoading}
                error={fileInfoQuery.isError ? fileInfoQuery.error : null}
                isDownloading={downloadMutation.isPending}
                onDownload={(file) => downloadMutation.mutate(file)}
              />
            </Modal>
          </>
        )}
      </Container>
    </Box>
  );
};

function FileEntry({
  entry,
  selected,
  onClick,
}: {
  entry: DirectoryEntry;
  selected: boolean;
  onClick: () => void;
}) {
  const disabled = !entry.isDirectory && !entry.fileUri;

  return (
    <UnstyledButton
      className={`${styles.entry} ${selected ? styles.selectedEntry : ""}`}
      onClick={onClick}
      disabled={disabled}
      aria-label={`${entry.isDirectory ? "フォルダ" : "ファイル"}: ${entry.name}`}
      aria-current={selected ? "true" : undefined}
    >
      <Group gap="sm" wrap="nowrap" className={styles.entryName}>
        {entry.isDirectory ? (
          <IconFolder size={19} className={styles.folderIcon} />
        ) : (
          <IconFile size={19} className={styles.fileIcon} />
        )}
        <Text size="sm" truncate title={entry.name}>
          {entry.name}
        </Text>
      </Group>
      <Text size="xs" c="dimmed" className={styles.dateColumn}>
        {entry.updatedAt ? formatDateTime(entry.updatedAt) : "—"}
      </Text>
      <Text size="xs" c="dimmed" ta="right" className={styles.sizeColumn}>
        {formatFileSize(entry.size)}
      </Text>
      <Text size="xs" c="dimmed" ta="right">
        {entry.isDirectory ? "フォルダ" : "ファイル"}
      </Text>
      {entry.isDirectory ? (
        <IconChevronRight size={16} className={styles.disclosureIcon} />
      ) : (
        <span aria-hidden="true" />
      )}
    </UnstyledButton>
  );
}

function SortHeader({
  label,
  column,
  activeColumn,
  direction,
  onClick,
  className,
  align = "left",
}: {
  label: string;
  column: SortColumn;
  activeColumn: SortColumn;
  direction: SortDirection;
  onClick: (column: SortColumn) => void;
  className?: string;
  align?: "left" | "right";
}) {
  const active = column === activeColumn;
  const sortDirection = direction === "asc" ? "昇順" : "降順";
  const SortIcon = direction === "asc" ? IconChevronUp : IconChevronDown;

  return (
    <UnstyledButton
      className={`${styles.sortButton} ${className ?? ""}`}
      onClick={() => onClick(column)}
      aria-label={`${label}で並べ替え${active ? `（${sortDirection}）` : ""}`}
      style={{ justifySelf: align === "right" ? "end" : "start" }}
    >
      <Text size="xs" fw={600} c="inherit">
        {label}
      </Text>
      {active && <SortIcon size={13} />}
    </UnstyledButton>
  );
}

function FileDetails({
  file,
  isLoading,
  error,
  isDownloading,
  onDownload,
}: {
  file: FileMetadata | undefined;
  isLoading: boolean;
  error: Error | null;
  isDownloading: boolean;
  onDownload: (file: FileMetadata) => void;
}) {
  return (
    <Box>
      {isLoading ? (
        <StatusMessage text="メタデータを読み込んでいます…" loading />
      ) : error ? (
        <StatusMessage
          text={`メタデータを取得できませんでした: ${getErrorMessage(error)}`}
          color="red"
        />
      ) : file ? (
        <>
          <Stack gap="sm" mb="lg">
            <MetadataRow label="名前" value={file.name} />
            <MetadataRow label="コンテンツタイプ" value={file.contentType} />
            <MetadataRow label="サイズ" value={formatFileSize(file.size)} />
            <MetadataRow
              label="更新日時"
              value={file.updatedAt ? formatDateTime(file.updatedAt) : "不明"}
            />
            <MetadataRow label="ファイル URI" value={file.fileUri} />
            <MetadataRow
              label="ファイルハッシュ"
              value={file.fileHash ?? "不明"}
            />
          </Stack>
          <Group justify="flex-end">
            <Button
              leftSection={<IconDownload size={16} />}
              loading={isDownloading}
              onClick={() => onDownload(file)}
            >
              ダウンロード
            </Button>
          </Group>
        </>
      ) : (
        <Text c="dimmed" size="sm">
          ファイルを選択するとメタデータが表示されます。
        </Text>
      )}
    </Box>
  );
}

function MetadataRow({ label, value }: { label: string; value: string }) {
  return (
    <Box>
      <Text size="xs" c="dimmed" mb={2}>
        {label}
      </Text>
      <Text size="sm" style={{ overflowWrap: "anywhere" }}>
        {value}
      </Text>
    </Box>
  );
}

function StatusMessage({
  text,
  color = "dimmed",
  loading = false,
}: {
  text: string;
  color?: string;
  loading?: boolean;
}) {
  return (
    <Group justify="center" gap="sm" py="xl">
      {loading && <Loader size="sm" />}
      <Text c={color} size="sm" ta="center">
        {text}
      </Text>
    </Group>
  );
}

function getErrorMessage(error: Error) {
  return error.message || "不明なエラー";
}

function formatFileSize(size: string) {
  const bytes = BigInt(size);
  const units = ["B", "KB", "MB", "GB", "TB"];
  let unitIndex = 0;
  let divisor = 1n;

  while (bytes >= divisor * 1024n && unitIndex < units.length - 1) {
    divisor *= 1024n;
    unitIndex += 1;
  }
  if (unitIndex === 0) return `${bytes} ${units[unitIndex]}`;

  const tenths = (bytes * 10n + divisor / 2n) / divisor;
  return `${tenths / 10n}.${tenths % 10n} ${units[unitIndex]}`;
}

function sortEntries(
  entries: DirectoryEntry[],
  column: SortColumn,
  direction: SortDirection,
) {
  const multiplier = direction === "asc" ? 1 : -1;

  return [...entries].sort((a, b) => {
    let comparison = 0;

    if (column === "kind") {
      comparison = Number(b.isDirectory) - Number(a.isDirectory);
    } else {
      if (a.isDirectory !== b.isDirectory) return a.isDirectory ? -1 : 1;

      switch (column) {
        case "name":
          comparison = a.name.localeCompare(b.name, "ja");
          break;
        case "updatedAt":
          if (!a.updatedAt || !b.updatedAt) {
            if (!a.updatedAt && !b.updatedAt) comparison = 0;
            else return !a.updatedAt ? 1 : -1;
          } else {
            comparison = Date.parse(a.updatedAt) - Date.parse(b.updatedAt);
          }
          break;
        case "size": {
          const aSize = BigInt(a.size);
          const bSize = BigInt(b.size);
          comparison = aSize < bSize ? -1 : aSize > bSize ? 1 : 0;
          break;
        }
      }
    }

    if (comparison === 0) comparison = a.name.localeCompare(b.name, "ja");
    return comparison * multiplier;
  });
}
