import { notifications } from "@mantine/notifications";
import i18n from "../i18n";

export const showInfoNotification = (message: string) => {
  notifications.show({
    title: i18n.t("通知"),
    message: message,
  });
};
