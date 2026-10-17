import { confirm, message } from "@tauri-apps/plugin-dialog";
import i18n from "../i18n";

export const showErrorDialog = async (msg: string) => {
  await message(msg.slice(0, 300), {
    title: i18n.t("エラー"),
    kind: "error",
  });
};

export const showConfirmDialog = async (
  msg: string,
  onConfirm: (ok: boolean) => void,
) => {
  const confirmation = await confirm(msg, {
    title: i18n.t("確認"),
    kind: "warning",
  });
  onConfirm(confirmation);
};
