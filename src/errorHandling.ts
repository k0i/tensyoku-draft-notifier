import { sendNotification } from "@tauri-apps/api/notification";

export const asyncErrorHandling = async (cb: Function) => {
        try {
                return await cb();
        } catch (e) {
                if (e instanceof Error) {
                        sendNotification({
                                title: "Error",
                                body: e.message,
                        });
                } else if (typeof e === "string") {
                        sendNotification({
                                title: "Error",
                                body: e,
                        });
                } else {
                        sendNotification({
                                title: "Error",
                                body: "Unknown error",
                        });
                }
        }
};
