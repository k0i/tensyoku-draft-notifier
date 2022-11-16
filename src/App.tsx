import { useState } from "react";
import { BaseDirectory, createDir, writeTextFile } from "@tauri-apps/api/fs";
import { appDataDir } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
        const [event, setEvent] = useState([]);
        const [id, setID] = useState("");
        const createDataFolder = async () => {
                const a = await appDataDir();
                console.log(a);
                await createDir("tensyoku-scraping", {
                        dir: BaseDirectory.AppData,
                        recursive: true,
                });
        };
        async function fetchEvents() {
                await createDataFolder();
                await writeTextFile(
                        "tensyoku-scraping/tensyoku-scraping.json",
                        "file contents",
                        {
                                dir: BaseDirectory.AppData,
                        }
                );
                setEvent(await invoke("fetch_event", { id }));
        }

        return (
                <div className="container">
                        <h1>転職Draft Notifier</h1>

                        <p>Your User ID</p>

                        <div className="row">
                                <div>
                                        <input
                                                id="greet-input"
                                                onChange={(e) => setID(e.currentTarget.value)}
                                                placeholder="Enter a name..."
                                        />
                                        <button type="button" onClick={() => fetchEvents()}>
                                                Start
                                        </button>
                                </div>
                        </div>
                        <p>{event.toString()}</p>
                </div>
        );
}

export default App;
