import { IChangeEvent } from "@rjsf/core";
import { commands, Index, OAuthClient, PluginInfo } from "../bindings";
import { RJSFSchema } from "@rjsf/utils";
import { useEffect, useState } from "react";
import {
  Button,
  Divider,
  Group,
  Loader,
  Paper,
  Select,
  Stack,
  Text,
} from "@mantine/core";
import { JsonSchemaForm } from "./JsonSchemaForm";
import { showErrorDialog } from "./Dialogs";
import { showInfoNotification } from "./Notifications";

interface PluginSelectorProps {
  label: string;
  saving: boolean;
  error: boolean;
  info: PluginInfo;
  selected: string | null;
  plugin_type: "fileExtractor" | "embedder" | "vectorStore";
  onSubmit: (
    event: IChangeEvent<any, RJSFSchema, any>,
    plugin_name: string,
  ) => void;
  initParams: any | null;
  children?: React.ReactNode;
}

export const PluginSelector = (props: PluginSelectorProps) => {
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
  } = props;
  const [selecting, setSelecting] = useState(selected);
  const [paramSchema, setParamSchema] = useState<any | null>(null);
  const [resetKey, setResetKey] = useState(0);
  const onCancel = () => {
    setSelecting(selected);
    setResetKey((prev) => prev + 1);
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
            {children}
            <Divider my="sm" />
            <JsonSchemaForm
              disabled={saving}
              key={resetKey}
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
};

interface FileSourceSelectorProps {
  label: string;
  info: PluginInfo;
  saving: boolean;
  error: boolean;
  index: Index;
  plugin_type: "fileSource";
  onSubmit: (
    event: IChangeEvent<any, RJSFSchema, any>,
    plugin_name: string,
    oauthClient: OAuthClient | null,
  ) => void;
  initParams: any | null;
  oauthClients: OAuthClient[];
}

export const FileSourceSelector = (props: FileSourceSelectorProps) => {
  const {
    label,
    info,
    index,
    saving,
    error,
    plugin_type,
    oauthClients,
    onSubmit,
  } = props;
  const fileSource = index?.fileSource;
  const savedOauthClient = oauthClients.find(
    (v) => v.id === fileSource?.oauthId,
  );
  const [selecting, setSelecting] = useState(fileSource?.name ?? null);
  const [paramSchema, setParamSchema] = useState<any | null>(null);
  const [resetKey, setResetKey] = useState(0);
  const [oauthId, setOauthId] = useState(fileSource?.oauthId ?? null);
  const [isTesting, setIsTesting] = useState(false);
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const cantChange = saving || isTesting || isAuthenticating;
  const isOAuthDirty = fileSource?.oauthId !== oauthId;
  const onCancel = () => {
    setSelecting(fileSource?.name ?? null);
    setResetKey((prev) => prev + 1);
    setOauthId(fileSource?.oauthId ?? null);
  };
  const onClickTest = async (plugin_name: string, params: any) => {
    setIsTesting(true);
    try {
      const result = await commands.testFileSource(
        index?.id,
        plugin_name,
        JSON.stringify(params),
      );
      if (result.status === "error") {
        await showErrorDialog(result.error);
      } else {
        showInfoNotification("ファイルソースのテストに成功しました");
      }
    } finally {
      setIsTesting(false);
    }
  };
  const onClickAuthenticate = async () => {
    setIsAuthenticating(true);
    try {
      if (!savedOauthClient) {
        return;
      }
      const result = await commands.doOauthAuthenticationIfNot(
        index.id,
        savedOauthClient,
        true,
      );
      if (result.status === "error") {
        await showErrorDialog(result.error);
      } else {
        showInfoNotification("OAuth認証に成功しました");
      }
    } finally {
      setIsAuthenticating(false);
    }
  };
  const onAbortOAuth = async () => {
    const oauthClient = oauthClients.find((v) => v.id === oauthId)?.clientId;
    if (oauthClient === undefined) return;
    commands.abortOauth(oauthClient);
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
            label={plugin_type}
            value={selecting}
            data={info.plugins}
            onChange={async (value, _option) => {
              if (value === null || value.length === 0) {
                return;
              }
              setSelecting(value);
            }}
            disabled={cantChange}
            error={error && "保存できませんでした。"}
            style={{ flex: 1 }}
          />
          <Group gap="xs" mt={24} wrap="nowrap">
            <Button variant="default" onClick={onCancel} disabled={cantChange}>
              リセット
            </Button>
          </Group>
        </Group>
        {paramSchema !== null && (
          <>
            <Select
              label={"OAuthクライアント"}
              value={oauthId}
              data={oauthClients.map((v) => ({ label: v.name, value: v.id }))}
              clearable
              disabled={cantChange}
              onChange={(v) => {
                setOauthId(v);
              }}
            />
            <Divider my="sm" />
            <JsonSchemaForm
              disabled={cantChange}
              key={resetKey}
              initParams={fileSource?.params}
              jsonSchema={paramSchema}
              onSubmit={(event) => {
                if (selecting === null) return;
                onSubmit(
                  event,
                  selecting,
                  oauthId
                    ? (oauthClients.find((v) => v.id === oauthId) ?? null)
                    : null,
                );
              }}
              submitButtonClosure={(getForm) => {
                return (_) => {
                  return (
                    <Group gap="xs" mt={24} wrap="nowrap">
                      <Button type="submit">{"保存"}</Button>
                      {index.fileSource?.oauthId &&
                        (isAuthenticating ? (
                          <Button variant="outline" onClick={onAbortOAuth}>
                            <Loader
                              size={"sm"}
                              style={{ marginRight: "10px" }}
                            />
                            認証を中断
                          </Button>
                        ) : (
                          <Button
                            variant="outline"
                            onClick={onClickAuthenticate}
                            disabled={isOAuthDirty || cantChange}
                          >
                            OAuth認証
                          </Button>
                        ))}
                      <Button
                        variant="outline"
                        disabled={isOAuthDirty || cantChange}
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
                    </Group>
                  );
                };
              }}
            />
          </>
        )}
      </Stack>
    </Paper>
  );
};
