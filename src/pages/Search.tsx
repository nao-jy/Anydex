import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Badge,
  Box,
  Button,
  Card,
  Container,
  Group,
  Loader,
  MultiSelect,
  Paper,
  Select,
  Stack,
  Text,
  TextInput,
  ThemeIcon,
} from "@mantine/core";
import { commands } from "../bindings";
import { unwrapCommandResult } from "../tauri";
import { PageHeader } from "../components/PageHeader";

export const Search = () => {
  return (
    <Box bg="var(--mantine-color-body)" mih="calc(100vh - 60px)" py={40}>
      <Container size="lg">
        <PageHeader pageName="検索" description="コンテンツを検索します" />
        <Card withBorder radius="md" padding="lg" mb="xl"></Card>
      </Container>
    </Box>
  );
};
