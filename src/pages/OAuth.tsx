import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Accordion, Box, Button, Card, Container, Group } from "@mantine/core";
import { commands } from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { PageHeader } from "../components/PageHeader";
import { showErrorDialog } from "../components/Dialogs";

export const OAuth = () => {
  const [open, setOpen] = useState<string[]>([]);
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
  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <Group justify="space-between" align="flex-end" mb={28}>
          <PageHeader
            pageName="OAuth設定"
            description="ファイルソースのための認証を設定します"
          />
          <Button size="md" onClick={() => {}}>
            ＋ 新しいOAuth
          </Button>
        </Group>
        <Card withBorder radius="md" padding="lg" mb="xl">
          <Accordion
            multiple
            value={open}
            onChange={setOpen}
            variant="separated"
            radius="md"
          ></Accordion>
        </Card>
      </Container>
    </Box>
  );
};
