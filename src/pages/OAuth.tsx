import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Accordion,
  Box,
  Button,
  Container,
  Group,
  Stack,
  TagsInput,
  Text,
  TextInput,
} from "@mantine/core";
import { commands, OAuthClient } from "../bindings";
import { PageHeader } from "../components/PageHeader";
import { showConfirmDialog, showErrorDialog } from "../components/Dialogs";

export const OAuth = () => {
  const [open, setOpen] = useState<string[]>([]);
  const queryClient = useQueryClient();
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
  const createOauth = useMutation({
    mutationFn: async () => {
      const result = await commands.createEmptyOauthClient();
      if (result.status === "error") {
        await showErrorDialog(result.error);
        throw new Error(result.error);
      }
      return result.data;
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["oauths"] }),
  });
  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <Group justify="space-between" align="flex-end" mb={28}>
          <PageHeader
            pageName="OAuth設定"
            description="ファイルソースのためのOAuth認証（PKCE）を設定します"
          />
          <Button
            size="md"
            onClick={() => {
              createOauth.mutate();
            }}
          >
            ＋ 新しいOAuth
          </Button>
        </Group>
        <Accordion
          multiple
          value={open}
          onChange={setOpen}
          variant="separated"
          radius="md"
        >
          {listOauthClient.data?.map((cl) => {
            return (
              <Accordion.Item value={cl.id.toString()} key={cl.id}>
                <Group gap={0} wrap="nowrap" align="stretch">
                  <Accordion.Control style={{ flex: 1 }}>
                    <Group justify="space-between" wrap="nowrap" pr="md">
                      <Box>
                        <Group gap="sm">
                          <Text fw={700} size="lg">
                            {cl.name}
                          </Text>
                        </Group>
                      </Box>
                    </Group>
                  </Accordion.Control>
                </Group>
                <Accordion.Panel>
                  <OAuthContent
                    data={cl}
                    onSave={async (data) => {
                      const result = await commands.updateOauthClient(data);
                      if (result.status === "error") {
                        await showErrorDialog(result.error);
                        throw new Error(result.error);
                      }
                      queryClient.invalidateQueries({ queryKey: ["oauths"] });
                    }}
                    onDelete={async () => {
                      const result = await commands.deleteOauthClient(cl.id);
                      if (result.status === "error") {
                        await showErrorDialog(result.error);
                        throw new Error(result.error);
                      }
                      queryClient.invalidateQueries({ queryKey: ["oauths"] });
                    }}
                  />
                </Accordion.Panel>
              </Accordion.Item>
            );
          })}
        </Accordion>
      </Container>
    </Box>
  );
};

interface OAuthContentProps {
  data: OAuthClient;
  onSave: (data: OAuthClient) => void;
  onDelete: (oauth_id: number) => void;
}

const OAuthContent = (props: OAuthContentProps) => {
  const { data, onSave, onDelete } = props;
  const [name, setName] = useState(data.name);
  const [cliendId, setClientId] = useState(data.clientId);
  const [authEndpoint, setAuthEndpoint] = useState(data.authorizationEndpoint);
  const [tokenEndpoint, setTokenEndpoint] = useState(data.tokenEndpoint);
  const [refreshEndpoint, setRefreshEndpoint] = useState(
    data.refreshEndpoint ?? "",
  );
  const [scopes, setScopes] = useState<string[]>(data.scopes as string[]);
  return (
    <Stack gap={"md"}>
      <TextInput
        label={"名前"}
        value={name}
        onChange={(v) => setName(v.currentTarget.value)}
        required
      />
      <TextInput
        label={"クライアントID"}
        value={cliendId}
        onChange={(v) => setClientId(v.currentTarget.value)}
        required
      />
      <TextInput
        label={"認証エンドポイント"}
        type="url"
        value={authEndpoint}
        onChange={(v) => setAuthEndpoint(v.currentTarget.value)}
        required
      />
      <TextInput
        label={"トークンエンドポイント"}
        type="url"
        value={tokenEndpoint}
        onChange={(v) => setTokenEndpoint(v.currentTarget.value)}
        required
      />
      <TextInput
        label={"リフレッシュエンドポイント"}
        type="url"
        value={refreshEndpoint}
        onChange={(v) => setRefreshEndpoint(v.currentTarget.value)}
      />
      <TagsInput
        label={"スコープ"}
        value={scopes}
        onChange={(v) => setScopes(v)}
        required
      />
      <Group gap="xs" mt={24} wrap="nowrap">
        <Button
          onClick={() => {
            const refr =
              refreshEndpoint.trim().length === 0
                ? null
                : refreshEndpoint.trim();
            const newData: OAuthClient = {
              id: data.id,
              name: name,
              clientId: cliendId,
              authorizationEndpoint: authEndpoint,
              tokenEndpoint: tokenEndpoint,
              refreshEndpoint: refr,
              scopes: scopes,
            };
            onSave(newData);
          }}
        >
          保存
        </Button>
        <Button
          color="red"
          onClick={async () => {
            await showConfirmDialog(
              "OAuthを削除します、よろしいですか？",
              async (ok) => {
                if (ok) {
                  onDelete(data.id);
                }
              },
            );
          }}
        >
          削除
        </Button>
      </Group>
    </Stack>
  );
};
