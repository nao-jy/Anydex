import { Box, Text } from "@mantine/core";
import { useTranslation } from "react-i18next";

interface DateInfoProps {
  label: string;
  value: Date | null;
}

export const DateInfo = (params: DateInfoProps) => {
  const { i18n } = useTranslation();
  const { label, value } = params;
  const localValue = value === null || Number.isNaN(value.getTime())
    ? "-"
    : value.toLocaleString(i18n.language);

  return (
    <Box>
      <Text size="xs" c="dimmed">
        {label}
      </Text>
      <Text size="sm">{localValue}</Text>
    </Box>
  );
};
