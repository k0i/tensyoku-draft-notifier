import { useEffect, useState } from "react";
import { sendNotification } from "@tauri-apps/api/notification";
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
        Spinner,
        Stack,
        Text,
} from "@chakra-ui/react";
import { MdOutlineInfo } from "react-icons/md";
import { bootUp } from "./start";
import { listen, emit, UnlistenFn } from "@tauri-apps/api/event";

function App() {
        // TODO: reRendering performance issue
        const [id, setID] = useState("");
        const [logState, setLogState] = useState({
                time: new Date().toLocaleTimeString(),
                newLogs: [] as string[],
                history: [] as string[],
        });
        useEffect(() => {
                const asyncBoot = async () => {
                        const config = await bootUp();
                        if (config) {
                                setID(config.id.toString());
                                setLogState((prev) => ({
                                        ...prev,
                                        history: config.logs,
                                }));
                        }
                };
                asyncBoot();
                let unlisten: UnlistenFn;
                async function fetchNewLog() {
                        unlisten = await listen("fetch_new_log", (event) => {
                                const { logs } = event.payload as { logs: string[] };
                                setLogState((prev) => ({
                                        time: new Date().toLocaleTimeString(),
                                        newLogs: logs,
                                        history: [...logs, ...prev.history],
                                }));
                                sendNotification({ title: "転職ドラフト", body: logs[0] });
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
                                        Tenshoku Draft Notifier
                                </Heading>
                        </Box>
                        <Box textAlign="center" py={6}>
                                {id === "" || logState.history.length === 0 ? (
                                        <Heading as="h2" size="md">
                                                Please Enter Your User ID
                                        </Heading>
                                ) : (
                                        <>
                                                <Heading as="h2" size="md">
                                                        Your current ID : {id}
                                                </Heading>
                                                <Heading as="h4" size="md">
                                                        Last updated at <Text as="em">{logState.time}</Text>
                                                </Heading>
                                        </>
                                )}
                        </Box>

                        <Stack direction="row" py={8} justifyContent="center">
                                <Input
                                        onChange={(e) => setID(e.currentTarget.value)}
                                        focusBorderColor="pink.400"
                                        w="30%"
                                />
                                <Button onClick={() => fetchEvents()} color="teal.400">
                                        {id === "" ? "Subscribe" : "Resubscribe"}
                                </Button>
                        </Stack>

                        {id !== "" && logState.history.length !== 0 && (
                                <Stack direction="row" py={10} justifyContent="center" mx="10%">
                                        <Spinner color="red.500" />
                                        <List spacing={3} alignSelf="center" justifySelf="center">
                                                <Heading as="h3" size="lg">
                                                        Recent Logs
                                                </Heading>
                                                {logState.newLogs.map((l) => {
                                                        return (
                                                                <ListItem key={l}>
                                                                        <ListIcon as={MdOutlineInfo} color="green.500" />
                                                                        {l}
                                                                </ListItem>
                                                        );
                                                })}
                                                <Divider py={2} my={8} borderColor="red.400" />
                                                <Heading as="h3" size="lg">
                                                        History
                                                </Heading>
                                                {logState.history.map((h, i) => {
                                                        return (
                                                                <ListItem key={h + i.toString()}>
                                                                        <ListIcon as={MdOutlineInfo} color="green.500" />
                                                                        {h}
                                                                </ListItem>
                                                        );
                                                })}
                                        </List>
                                </Stack>
                        )}
                </Container>
        );
}

export default App;
