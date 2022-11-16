import { useEffect, useState } from "react";
import { sendNotification } from "@tauri-apps/api/notification";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import {
        Box,
        Button,
        Container,
        Heading,
        Input,
        List,
        ListIcon,
        ListItem,
        Stack,
} from "@chakra-ui/react";
import { MdOutlineInfo } from "react-icons/md";
import { bootUp } from "./start";
import { appWindow } from "@tauri-apps/api/window";

function App() {
        const [event, setEvent] = useState([]);
        const [id, setID] = useState("");
        const [logs, setLogs] = useState<string[]>([]);
        useEffect(() => {
                const asyncBoot = async () => {
                        const config = await bootUp();
                        if (config) {
                                setID(config.id.toString());
                                setLogs(config.logs);
                        }
                };
                asyncBoot();
        }, []);
        async function fetchEvents() {
                appWindow.minimize();
                setEvent(await invoke("fetch_event", { id }));
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
                                        {event.map((e) => {
                                                return (
                                                        <ListItem key={e}>
                                                                <ListIcon as={MdOutlineInfo} color="green.500" />
                                                                {e}
                                                        </ListItem>
                                                );
                                        })}
                                </List>
                        </Stack>
                </Container>
        );
}

export default App;
