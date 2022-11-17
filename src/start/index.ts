import { BaseDirectory, readTextFile } from "@tauri-apps/api/fs";
import {
        isPermissionGranted,
        requestPermission,
        sendNotification,
} from "@tauri-apps/api/notification";
import { APP_NAME, Config } from "../const";
import { createDataFolder, readFromFile } from "../fs";

export const bootUp = async () => {
        let permissionGranted = await isPermissionGranted();
        if (!permissionGranted) {
                const permission = await requestPermission();
                permissionGranted = permission === "granted";
        }
        const isInitial = await isInitialBoot();
        if (isInitial) {
                await createDataFolder();
                return;
        }

        try {
                const id = JSON.parse(
                        await readFromFile("tensyoku-scraping/user_id")
                ) as number;
                const logs = (await readFromFile("tensyoku-scraping/logs"))
                        .split("\n")
                        .filter((l: string) => l !== "");
                logs.reverse();
                return { id, logs } as Config;
        } catch (e) {
                sendNotification({
                        title: "Unexpected Error",
                        body: "failed to parse tensyoku-scraping.json.It is unexpected.",
                });
        }
};

const isInitialBoot = async () => {
        const path = `${APP_NAME}/user_id`;
        try {
                await readTextFile(path, {
                        dir: BaseDirectory.AppData,
                });
                return false;
        } catch {
                return true;
        }
};
