import { confirm, message } from "@tauri-apps/plugin-dialog";

export const showErrorDialog = async (msg: string) => {
  await message(msg, { title: "Error", kind: "error" });
};

export const showConfirmDialog = async (
  msg: string,
  onConfirm: (ok: boolean) => void,
) => {
  const confirmation = await confirm(msg, {
    title: "Confirm",
    kind: "warning",
  });
  onConfirm(confirmation);
};

export const showInfoDialog = async (msg: string) => {
  await message(msg, { title: "Info", kind: "info" });
};
