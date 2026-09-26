import { Box, Text, Title } from "@mantine/core";

interface PageHeaderProps {
  pageName: string;
  description: string;
}

export const PageHeader = (props: PageHeaderProps) => {
  const { pageName, description } = props;
  return (
    <Box mb={28}>
      <Title order={1} mt={4}>
        {pageName}
      </Title>
      <Text c="dimmed" mt={6}>
        {description}
      </Text>
    </Box>
  );
};
