import {
        BaseDirectory,
        createDir,
        readTextFile,
        writeTextFile,
} from "@tauri-apps/api/fs";
import { asyncErrorHandling } from "../errorHandling";

export const createDataFolder = async () => {
        await createDir("tensyoku-scraping", {
                dir: BaseDirectory.AppData,
                recursive: true,
        });
};

export const writeToFile = async (path: string, contents: string) => {
        await writeTextFile(path, contents, {
                dir: BaseDirectory.AppData,
        });
};

export const readFromFile = async (path: string) => {
        return await asyncErrorHandling(async () =>
                readTextFile(path, {
                        dir: BaseDirectory.AppData,
                })
        );
};
