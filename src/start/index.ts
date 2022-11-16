import { BaseDirectory, readTextFile } from "@tauri-apps/api/fs";
import {
        isPermissionGranted,
        requestPermission,
        sendNotification,
} from "@tauri-apps/api/notification";
import { APP_NAME, Config } from "../const";
import { createDataFolder, readFromFile, writeToFile } from "../fs";

export const bootUp = async () => {
        let permissionGranted = await isPermissionGranted();
        if (!permissionGranted) {
                const permission = await requestPermission();
                permissionGranted = permission === "granted";
        }
        const isInitial = await isInitialBoot();
        if (isInitial) {
                await createDataFolder();
                await writeToFile("tensyoku-scraping/tensyoku-scraping.json", "");
                return;
        }

        try {
                return JSON.parse(
                        await readFromFile("tensyoku-scraping/tensyoku-scraping.json")
                ) as Config;
        } catch (e) {
                sendNotification({
                        title: "Unexpected Error",
                        body: "failed to parse tensyoku-scraping.json.It is unexpected.",
                });
        }
};

const isInitialBoot = async () => {
        const path = `${APP_NAME}/tensyoku-scraping.json`;
        try {
                await readTextFile(path, {
                        dir: BaseDirectory.AppData,
                });
                return false;
        } catch {
                return true;
        }
};
