import { Button, Group, Text, Textarea, TextInput } from "@mantine/core";
import { useState } from "react";
import { useTranslation } from "react-i18next";

interface EditableTextInputProps {
  label: string;
  saving: boolean;
  error: boolean;
  value: string;
  emptyValue?: string;
  onSave: (newValue: string) => void;
}
export const EditableTextInput = (props: EditableTextInputProps) => {
  const { t } = useTranslation();
  const { onSave, saving, error, value, emptyValue, label } = props;
  const [isEditting, setIsEditting] = useState(false);
  const [newValue, setNewValue] = useState(value);

  const save = () => {
    const nextValue = newValue.trim();
    if (!nextValue) {
      return;
    }
    onSave(nextValue);
    setIsEditting(false);
  };

  const onCancel = () => {
    setIsEditting(false);
  };

  if (isEditting) {
    return (
      <Group align="flex-start" wrap="nowrap">
        <TextInput
          label={label}
          value={newValue}
          onChange={(event) => setNewValue(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Escape") onCancel();
          }}
          disabled={saving}
          error={error && t("保存できませんでした。")}
          style={{ flex: 1 }}
        />
        <Group gap="xs" mt={24} wrap="nowrap">
          <Button variant="default" onClick={onCancel} disabled={saving}>
            {t("キャンセル")}
          </Button>
          <Button onClick={save} loading={saving}>
            {t("保存")}
          </Button>
        </Group>
      </Group>
    );
  }

  return (
    <Group justify="space-between" wrap="nowrap">
      <Text size="sm" c="dimmed" style={{ whiteSpace: "pre-wrap" }}>
        {value.length === 0 ? (emptyValue ?? "") : value}
      </Text>

      <Button
        variant="subtle"
        size="compact-sm"
        onClick={() => setIsEditting(true)}
        style={{ flexShrink: "0" }}
      >
        {t("変更")}
      </Button>
    </Group>
  );
};

interface EditableTextAreaProps {
  label: string;
  saving: boolean;
  error: boolean;
  value: string;
  emptyValue?: string;
  row?: number;
  onSave: (newValue: string) => void;
}
export const EditableTextArea = (props: EditableTextAreaProps) => {
  const { t } = useTranslation();
  const { onSave, saving, error, value, emptyValue, label, row } = props;
  const [isEditting, setIsEditting] = useState(false);
  const [newValue, setNewValue] = useState(value);

  const save = () => {
    const nextValue = newValue.trim();
    if (!nextValue) {
      return;
    }
    onSave(nextValue);
    setIsEditting(false);
  };

  const onCancel = () => {
    setIsEditting(false);
  };

  if (isEditting) {
    return (
      <Group align="flex-start" wrap="nowrap">
        <Textarea
          label={label}
          value={newValue}
          onChange={(event) => setNewValue(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Escape") onCancel();
          }}
          disabled={saving}
          error={error && t("保存できませんでした。")}
          style={{ flex: 1 }}
          rows={row}
        />
        <Group gap="xs" mt={24} wrap="nowrap">
          <Button variant="default" onClick={onCancel} disabled={saving}>
            {t("キャンセル")}
          </Button>
          <Button onClick={save} loading={saving}>
            {t("保存")}
          </Button>
        </Group>
      </Group>
    );
  }

  return (
    <Group justify="space-between" wrap="nowrap">
      <Text size="sm" c="dimmed" style={{ whiteSpace: "pre-wrap" }}>
        {value.length === 0 ? (emptyValue ?? "") : value}
      </Text>

      <Button
        variant="subtle"
        size="compact-sm"
        onClick={() => setIsEditting(true)}
        style={{ flexShrink: "0" }}
      >
        {t("変更")}
      </Button>
    </Group>
  );
};
