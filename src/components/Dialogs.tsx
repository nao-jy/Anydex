import { message } from "@tauri-apps/plugin-dialog";

export const showErrorDialog = async (msg: string) => {
  await message(msg, { title: "Error", kind: "error" });
};
