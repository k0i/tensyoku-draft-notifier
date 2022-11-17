import { useEffect, useState } from "react";
import { sendNotification } from "@tauri-apps/api/notification";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import {
        Box,
        Button,
        Container,
        Divider,
        Heading,
        Input,
        List,
        ListIcon,
        ListItem,
        Stack,
} from "@chakra-ui/react";
import { MdOutlineInfo } from "react-icons/md";
import { bootUp } from "./start";
import { listen, emit } from "@tauri-apps/api/event";

function App() {
        const [id, setID] = useState("");
        const [newLogs, setNewLogs] = useState<string[]>([]);
        const [history, setHistory] = useState<string[]>([]);
        useEffect(() => {
                const asyncBoot = async () => {
                        const config = await bootUp();
                        if (config) {
                                setID(config.id.toString());
                                setHistory(config.logs);
                        }
                };
                asyncBoot();
                let unlisten: any;
                async function fetchNewLog() {
                        unlisten = await listen("fetch_new_log", (event) => {
                                const { logs } = event.payload as { logs: string[] };
                                sendNotification({ title: "転職ドラフト", body: logs[0] });
                                setHistory((prev) => [...logs, ...prev]);
                                setNewLogs(logs);
                        });
                }
                fetchNewLog();
                return () => {
                        if (unlisten) {
                                unlisten();
                        }
                };
        }, []);
        async function fetchEvents() {
                emit("input_user_id", id);
                //setEvent(await invoke("manual_fetch_new_log", { id }));
                sendNotification({ title: "転職ドラフト", body: "new notification" });
        }

        return (
                <Container
                        py={30}
                        minW="100%"
                        alignItems="center"
                        justifyContent="center"
                        justifyItems="center"
                >
                        <Box alignItems="center">
                                <Heading as="h1" size="2xl">
                                        転職Draft Notifier
                                </Heading>
                        </Box>
                        <Box textAlign="center" py={6}>
                                <Heading as="h2" size="md">
                                        Please Enter Your User ID
                                </Heading>
                        </Box>

                        <Stack direction="row" py={8} justifyContent="center">
                                <Input
                                        onChange={(e) => setID(e.currentTarget.value)}
                                        focusBorderColor="pink.400"
                                        w="30%"
                                />
                                <Button onClick={() => fetchEvents()} color="teal.400">
                                        Subscribe
                                </Button>
                        </Stack>

                        <Stack direction="row" py={10} justifyContent="center" mx="10%">
                                <List spacing={3} alignSelf="center" justifySelf="center">
                                        {newLogs.map((l) => {
                                                return (
                                                        <ListItem key={l}>
                                                                <ListIcon as={MdOutlineInfo} color="green.500" />
                                                                {l}
                                                        </ListItem>
                                                );
                                        })}
                                        <Divider />
                                        <Heading>History</Heading>
                                        {history.map((h) => {
                                                return (
                                                        <ListItem key={h}>
                                                                <ListIcon as={MdOutlineInfo} color="green.500" />
                                                                {h}
                                                        </ListItem>
                                                );
                                        })}
                                </List>
                        </Stack>
                </Container>
        );
}

export default App;
