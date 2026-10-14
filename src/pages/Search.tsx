import { useMemo, useRef, useState, type FormEvent } from "react";
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
  SegmentedControl,
  Select,
  Stack,
  Text,
  TextInput,
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
  IconPhoto,
  IconSearch,
  IconX,
} from "@tabler/icons-react";
import { save } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type DirectoryEntry,
  type FileMetadata,
  type SemanticSearchQuery,
  type SearchMatch,
} from "../bindings";
import { showErrorDialog } from "../components/Dialogs";
import { showInfoNotification } from "../components/Notifications";
import { PageHeader } from "../components/PageHeader";
import { formatDateTime } from "../components/utils";
import { unwrapCommandResult } from "../tauri";
import styles from "./Search.module.css";

type SortColumn = "name" | "updatedAt" | "size" | "kind";
type SortDirection = "asc" | "desc";
type SearchMode = "keyword" | "semantic";
type SearchScope = "current" | "index";
type SemanticInputType = "text" | "image";
type SubmittedSearch =
  | {
      id: string;
      indexId: string;
      mode: "keyword";
      query: string;
      label: string;
      target: string;
    }
  | {
      id: string;
      indexId: string;
      mode: "semantic";
      query: SemanticSearchQuery;
      label: string;
      target: string;
    };

export const Search = () => {
  const [selectedIndexId, setSelectedIndexId] = useState<string | null>(null);
  const [directoryPath, setDirectoryPath] = useState("/");
  const [selectedFileUri, setSelectedFileUri] = useState<string | null>(null);
  const [sortColumn, setSortColumn] = useState<SortColumn>("name");
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");
  const [searchMode, setSearchMode] = useState<SearchMode>("keyword");
  const [semanticInputType, setSemanticInputType] =
    useState<SemanticInputType>("text");
  const [searchScope, setSearchScope] = useState<SearchScope>("current");
  const [searchInput, setSearchInput] = useState("");
  const [imageQuery, setImageQuery] = useState<{
    name: string;
    base64: string;
  } | null>(null);
  const [imageError, setImageError] = useState<string | null>(null);
  const [submittedSearch, setSubmittedSearch] = useState<SubmittedSearch | null>(
    null,
  );
  const imageInputRef = useRef<HTMLInputElement>(null);

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

  const searchQuery = useQuery({
    queryKey: [
      "search",
      submittedSearch?.id,
    ],
    queryFn: async (): Promise<SearchMatch[]> => {
      if (!submittedSearch) return [];
      if (submittedSearch.mode === "keyword") {
        const result = await unwrapCommandResult(
          commands.grep(
            submittedSearch.indexId,
            submittedSearch.query,
            submittedSearch.target,
          ),
        );
        return result.matches;
      }
      const result = await unwrapCommandResult(
        commands.semanticSearch(
          submittedSearch.indexId,
          submittedSearch.query,
          submittedSearch.target,
        ),
      );
      return result.matches;
    },
    enabled: submittedSearch !== null,
  });

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
    setSubmittedSearch(null);
  };
  const submitSearch = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!indexId) return;
    const target = searchScope === "index" ? "/" : directoryPath;
    const id = crypto.randomUUID();

    if (searchMode === "keyword") {
      const query = searchInput.trim();
      if (!query) return;
      setSubmittedSearch({
        id,
        indexId,
        mode: "keyword",
        query,
        label: query,
        target,
      });
      return;
    }

    if (semanticInputType === "image") {
      if (!imageQuery) return;
      setSubmittedSearch({
        id,
        indexId,
        mode: "semantic",
        query: { type: "imageBase64", value: imageQuery.base64 },
        label: `画像: ${imageQuery.name}`,
        target,
      });
      return;
    }

    const query = searchInput.trim();
    if (!query) return;
    setSubmittedSearch({
      id,
      indexId,
      mode: "semantic",
      query: { type: "text", value: query },
      label: query,
      target,
    });
  };
  const chooseImage = (file: File | undefined) => {
    setImageError(null);
    if (!file) return;
    setImageQuery(null);
    if (!file.type.startsWith("image/")) {
      setImageError("画像ファイルを選択してください。");
      return;
    }

    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result !== "string") {
        setImageQuery(null);
        setImageError("画像ファイルを読み込めませんでした。");
        return;
      }
      const base64 = reader.result.split(",")[1];
      if (!base64) {
        setImageQuery(null);
        setImageError("画像ファイルを読み込めませんでした。");
        return;
      }
      setImageQuery({ name: file.name, base64 });
    };
    reader.onerror = () => {
      setImageQuery(null);
      setImageError("画像ファイルを読み込めませんでした。");
    };
    reader.readAsDataURL(file);
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
  const canSubmitSearch =
    searchMode === "keyword" || semanticInputType === "text"
      ? Boolean(searchInput.trim())
      : imageQuery !== null;

  return (
    <Box bg="var(--mantine-color-body)" py={20}>
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
                setSubmittedSearch(null);
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
            <Card withBorder radius="md" padding="md" mb="md">
              <form onSubmit={submitSearch}>
                <Stack gap="sm">
                  <Group justify="space-between" align="center" wrap="wrap">
                    <SegmentedControl
                      value={searchMode}
                      onChange={(value) => {
                        if (value === "keyword" || value === "semantic") {
                          setSearchMode(value);
                        }
                      }}
                      data={[
                        { label: "キーワード検索", value: "keyword" },
                        { label: "意味検索", value: "semantic" },
                      ]}
                    />
                    {searchMode === "semantic" && (
                      <SegmentedControl
                        value={semanticInputType}
                        onChange={(value) => {
                          if (value === "text" || value === "image") {
                            setSemanticInputType(value);
                          }
                        }}
                        data={[
                          { label: "テキスト", value: "text" },
                          { label: "画像", value: "image" },
                        ]}
                      />
                    )}
                    <Select
                      aria-label="検索範囲"
                      value={searchScope}
                      onChange={(value) => {
                        if (value === "current" || value === "index") {
                          setSearchScope(value);
                        }
                      }}
                      data={[
                        {
                          value: "current",
                          label: `このフォルダ (${directoryPath})`,
                        },
                        { value: "index", label: "インデックス全体" },
                      ]}
                      w={{ base: "100%", sm: 260 }}
                    />
                  </Group>
                  <Group align="flex-start" wrap="nowrap">
                    {searchMode === "semantic" &&
                    semanticInputType === "image" ? (
                      <Group
                        gap="xs"
                        wrap="nowrap"
                        style={{ flex: 1, minWidth: 0 }}
                      >
                        <input
                          ref={imageInputRef}
                          type="file"
                          accept="image/*"
                          hidden
                          onChange={(event) => {
                            chooseImage(event.currentTarget.files?.[0]);
                            event.currentTarget.value = "";
                          }}
                        />
                        <Button
                          type="button"
                          variant="default"
                          leftSection={<IconPhoto size={16} />}
                          onClick={() => imageInputRef.current?.click()}
                        >
                          画像を選択
                        </Button>
                        {imageQuery ? (
                          <Group gap={4} wrap="nowrap" style={{ minWidth: 0 }}>
                            <Text size="sm" truncate title={imageQuery.name}>
                              {imageQuery.name}
                            </Text>
                            <ActionIcon
                              variant="subtle"
                              color="gray"
                              aria-label="選択した画像を解除"
                              onClick={() => {
                                setImageQuery(null);
                                setImageError(null);
                              }}
                            >
                              <IconX size={16} />
                            </ActionIcon>
                          </Group>
                        ) : (
                          <Text size="sm" c="dimmed">
                            画像を選択してください
                          </Text>
                        )}
                      </Group>
                    ) : (
                      <TextInput
                        aria-label="検索語"
                        placeholder={
                          searchMode === "keyword"
                            ? "本文から探すキーワードを入力"
                            : "例: 契約更新の条件について書かれた文書"
                        }
                        leftSection={<IconSearch size={16} />}
                        value={searchInput}
                        onChange={(event) =>
                          setSearchInput(event.currentTarget.value)
                        }
                        style={{ flex: 1 }}
                      />
                    )}
                    <Button
                      type="submit"
                      leftSection={<IconSearch size={16} />}
                      disabled={!canSubmitSearch}
                      loading={searchQuery.isFetching}
                    >
                      検索
                    </Button>
                  </Group>
                  {imageError && (
                    <Text size="xs" c="red">
                      {imageError}
                    </Text>
                  )}
                  <Text size="xs" c="dimmed">
                    {searchMode === "keyword"
                      ? "本文に入力したキーワードを含む箇所を検索します。"
                      : semanticInputType === "image"
                        ? "画像の内容に近いファイルを検索します。"
                        : "言葉が完全に一致しなくても、内容が近い箇所を検索します。"}
                  </Text>
                </Stack>
              </form>
            </Card>

            <Card withBorder radius="md" padding="sm" mb="md">
              <Group justify="space-between" wrap="nowrap">
                {submittedSearch ? (
                  <Group gap="xs" wrap="nowrap" style={{ minWidth: 0 }}>
                    <Text size="sm" fw={600} truncate>
                      {submittedSearch.mode === "keyword"
                        ? "キーワード検索"
                        : "意味検索"}
                      ：「{submittedSearch.label}」
                    </Text>
                    <Text size="xs" c="dimmed" style={{ flexShrink: 0 }}>
                      対象:{" "}
                      {submittedSearch.target === "/"
                        ? "インデックス全体"
                        : submittedSearch.target}
                    </Text>
                    <Button
                      variant="subtle"
                      size="compact-sm"
                      onClick={() => setSubmittedSearch(null)}
                    >
                      一覧に戻る
                    </Button>
                  </Group>
                ) : (
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
                )}
                <Text size="sm" c="dimmed" style={{ flexShrink: 0 }}>
                  {submittedSearch
                    ? searchQuery.isFetching
                      ? "検索中…"
                      : `${searchQuery.data?.length ?? 0} 件`
                    : `${entries.length} 件`}
                </Text>
              </Group>
            </Card>

            {submittedSearch ? (
              <Card withBorder radius="md" padding={0} style={{ minWidth: 0 }}>
                {searchQuery.isFetching ? (
                  <StatusMessage text="検索しています…" loading />
                ) : searchQuery.isError ? (
                  <StatusMessage
                    text={`検索できませんでした: ${getErrorMessage(searchQuery.error)}`}
                    color="red"
                  />
                ) : searchQuery.data?.length ? (
                  <Stack gap={0} p="sm">
                    {searchQuery.data.map((match, index) => (
                      <SearchResult
                        key={`${match.file.fileUri}-${index}`}
                        match={match}
                        onClick={() => setSelectedFileUri(match.file.fileUri)}
                      />
                    ))}
                  </Stack>
                ) : (
                  <Stack align="center" justify="center" gap="xs" py={48}>
                    <ThemeIcon
                      variant="light"
                      color="gray"
                      size={42}
                      radius="xl"
                    >
                      <IconSearch size={22} />
                    </ThemeIcon>
                    <Text size="sm" c="dimmed">
                      該当するファイルはありません
                    </Text>
                  </Stack>
                )}
              </Card>
            ) : (
              <Card withBorder radius="md" padding={0} style={{ minWidth: 0 }}>
                {directoryQuery.isLoading ? (
                  <StatusMessage
                    text="ファイル一覧を読み込んでいます…"
                    loading
                  />
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
            )}

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

function SearchResult({
  match,
  onClick,
}: {
  match: SearchMatch;
  onClick: () => void;
}) {
  return (
    <UnstyledButton
      className={styles.searchResult}
      onClick={onClick}
      aria-label={`ファイル情報を表示: ${match.file.name}`}
    >
      <Group gap="xs" wrap="nowrap" mb={4}>
        <IconFile size={17} className={styles.fileIcon} />
        <Text size="sm" fw={600} truncate title={match.file.name}>
          {match.file.name}
        </Text>
      </Group>
      <Text
        size="xs"
        c="dimmed"
        lineClamp={1}
        mb={4}
        title={match.file.fileUri}
      >
        {match.file.fileUri}
      </Text>
      <Text size="sm" lineClamp={3}>
        {match.content}
      </Text>
    </UnstyledButton>
  );
}

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
