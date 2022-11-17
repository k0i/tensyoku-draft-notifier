import { ChakraProvider } from "@chakra-ui/react";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./style.css";
import { extendTheme } from "@chakra-ui/react";

const theme = extendTheme({
        styles: {
                global: {
                        "html,body": {
                                backgroundColor: "gray.800",
                                color: "gray.100",
                                height: "100vh",
                        },
                },
        },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
        <React.StrictMode>
                <ChakraProvider theme={theme}>
                        <App />
                </ChakraProvider>
        </React.StrictMode>
);
