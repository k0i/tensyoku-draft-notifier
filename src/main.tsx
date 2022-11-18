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
                                color: "gray.100",
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
