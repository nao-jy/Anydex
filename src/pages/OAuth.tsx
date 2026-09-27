import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Accordion,
  Box,
  Button,
  Container,
  Group,
  Paper,
  Stack,
  Text,
} from "@mantine/core";
import { commands } from "../bindings";
import { PageHeader } from "../components/PageHeader";
import { showErrorDialog } from "../components/Dialogs";

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
            description="ファイルソースのための認証を設定します"
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
                  <Paper withBorder p="md" radius="md">
                    <Stack gap={"md"}></Stack>
                  </Paper>
                </Accordion.Panel>
              </Accordion.Item>
            );
          })}
        </Accordion>
      </Container>
    </Box>
  );
};
