import { Button, Input } from "@mantine/core";
import { getDefaultRegistry, withTheme } from "@rjsf/core";
import { Theme as MantineTheme } from "@rjsf/mantine";
import { FieldProps, SubmitButtonProps } from "@rjsf/utils";
import { customizeValidator } from "@rjsf/validator-ajv8";
import { JsonEditor } from "json-edit-react";
import Ajv2020 from "ajv/dist/2020";

const DefaultObjectField = getDefaultRegistry().fields.ObjectField;

export const CustomObjectField = (props: FieldProps) => {
  const {
    schema,
    uiSchema,
    name,
    formData,
    onChange,
    required,
    disabled,
    readonly,
    fieldPathId,
  } = props;

  const hasMembers = Object.keys(schema.properties ?? {}).length > 0;

  // メンバー定義がある場合は標準の ObjectField に任せる
  if (hasMembers && !schema.additionalProperties) {
    return <DefaultObjectField {...props} />;
  }

  // メンバー定義が 0 の場合は JSON Editor
  const title =
    (uiSchema?.["ui:title"] as string | undefined) ?? schema.title ?? name;

  return (
    <Input.Wrapper
      label={title}
      description={schema.description}
      required={required}
    >
      <JsonEditor
        data={formData ?? {}}
        setData={(newData) => onChange(newData as any, fieldPathId.path)}
        minWidth={"100%"}
        restrictEdit={disabled || readonly}
      />
    </Input.Wrapper>
  );
};

const SubmitButton = (_: SubmitButtonProps) => {
  return <Button type="submit">パラメータを保存</Button>;
};

export interface JsonSchemaFormParams {
  jsonSchema: any;
}

export const JsonSchemaForm = (props: JsonSchemaFormParams) => {
  const { jsonSchema } = props;
  const Form = withTheme(MantineTheme);
  const validator = customizeValidator({
    AjvClass: Ajv2020,
  });

  return (
    <Form
      schema={jsonSchema}
      validator={validator}
      fields={{
        ObjectField: CustomObjectField,
      }}
      templates={{
        ButtonTemplates: {
          SubmitButton: SubmitButton,
        },
      }}
      onSubmit={(data) => {
        console.log(data);
      }}
    ></Form>
  );
};
