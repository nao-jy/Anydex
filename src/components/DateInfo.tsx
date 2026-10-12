import { Box, Text } from "@mantine/core";

interface DateInfoProps {
  label: string;
  value: Date | null;
}

export const DateInfo = (params: DateInfoProps) => {
  const { label, value } = params;
  const localValue = value === null || Number.isNaN(value.getTime())
    ? "-"
    : value.toLocaleString();

  return (
    <Box>
      <Text size="xs" c="dimmed">
        {label}
      </Text>
      <Text size="sm">{localValue}</Text>
    </Box>
  );
};
