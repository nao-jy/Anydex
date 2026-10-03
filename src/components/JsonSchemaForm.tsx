import { Button, Input } from "@mantine/core";
import Form, { getDefaultRegistry, IChangeEvent, withTheme } from "@rjsf/core";
import { Theme as MantineTheme } from "@rjsf/mantine";
import { FieldProps, RJSFSchema, SubmitButtonProps } from "@rjsf/utils";
import { customizeValidator } from "@rjsf/validator-ajv8";
import { JsonEditor } from "json-edit-react";
import Ajv2020 from "ajv/dist/2020";
import Ajv2019 from "ajv/dist/2019";
import { useMemo, useRef } from "react";

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
        allowEdit={!(disabled || readonly)}
      />
    </Input.Wrapper>
  );
};

const SubmitButtonDefault = (_: SubmitButtonProps) => {
  return <Button type="submit">保存</Button>;
};

export interface JsonSchemaFormParams {
  jsonSchema: any;
  initParams: any | null;
  onSubmit: (event: IChangeEvent<any, RJSFSchema, any>) => void;
  disabled?: boolean;
  submitButtonClosure?: (
    getForm: () => Form<any, RJSFSchema, any> | null,
  ) => React.ComponentType<SubmitButtonProps>;
}

export const JsonSchemaForm = (props: JsonSchemaFormParams) => {
  const { jsonSchema, onSubmit, initParams, submitButtonClosure, disabled } =
    props;
  const Form = withTheme(MantineTheme);
  const validator = customizeValidator({
    AjvClass: String(jsonSchema.$schema).includes("draft/2020")
      ? Ajv2020
      : Ajv2019,
  });
  const formRef = useRef<Form<any, RJSFSchema, any>>(null);
  const submitButtonClosureRef = useRef(submitButtonClosure);
  submitButtonClosureRef.current = submitButtonClosure;
  const CustomSubmitButton = useMemo(() => {
    const getForm = () => formRef.current;
    return (buttonProps: SubmitButtonProps) => {
      const SubmitButton =
        submitButtonClosureRef.current?.(getForm) ?? SubmitButtonDefault;
      return <SubmitButton {...buttonProps} />;
    };
  }, []);

  return (
    <Form
      ref={formRef}
      disabled={disabled}
      formData={initParams ?? undefined}
      schema={jsonSchema}
      validator={validator}
      fields={{
        ObjectField: CustomObjectField,
      }}
      templates={{
        ButtonTemplates: {
          SubmitButton: CustomSubmitButton,
        },
      }}
      onSubmit={(data) => {
        onSubmit(data);
      }}
    ></Form>
  );
};
