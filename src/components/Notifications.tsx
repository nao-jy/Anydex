import { notifications } from "@mantine/notifications";

export const showInfoNotification = (message: string) => {
  notifications.show({
    title: "通知",
    message: message,
  });
};
