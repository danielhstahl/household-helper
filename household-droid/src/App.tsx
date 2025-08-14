import "./App.css";
import { useState } from "react";
import AppBarDroid from "./components/AppBar";
import AgentSelection from "./components/AgentSelection";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import Toolbar from "@mui/material/Toolbar";
import CssBaseline from "@mui/material/CssBaseline";
import Container from "@mui/material/Container";
import Chat from "./components/Chat";
import Output, { DialogEnum, type Message } from "./components/Output";
import { sendQuery } from "./services/Api";
const INIT_MESSAGE = {
  persona: DialogEnum.It,
  text: "Please start chatting!",
  id: 0,
};
interface MessagesAndText {
  messages: Message[];
  latestText: string;
}
const initMessageAndText = {
  messages: [INIT_MESSAGE],
  latestText: "",
};
function App() {
  const theme = createTheme();
  const [{ messages, latestText }, setMessages] =
    useState<MessagesAndText>(initMessageAndText);
  const [isWaiting, setIsWaiting] = useState(false);
  return (
    <ThemeProvider theme={theme}>
      <Container maxWidth={false} style={{ minHeight: "100%" }}>
        {/*Toolbar only here to push down below app bar*/}
        <Toolbar />
        <CssBaseline />
        <AppBarDroid />
        <AgentSelection />
        <Output
          messages={messages}
          latestText={latestText}
          //agentType="helper"
          isWaiting={isWaiting}
        />
        <Chat
          agentType="helper"
          onEnter={(v) => {
            setMessages((state) => ({
              latestText: state.latestText,
              messages: [
                ...state.messages,
                { persona: DialogEnum.Me, text: v, id: state.messages.length },
              ],
            }));
            sendQuery(v).then(async (r) => {
              let done = false;
              let value;
              const dec = new TextDecoder();
              while (!done) {
                ({ value, done } = await r.read());
                const strVal = dec.decode(value, { stream: true });
                setMessages((state) => ({
                  latestText: state.latestText + strVal,
                  messages: state.messages,
                }));
                //setLatestText((v) => v + strVal);
              }
              setMessages((state) => ({
                latestText: "",
                messages: [
                  ...state.messages,
                  {
                    persona: DialogEnum.It,
                    text: state.latestText,
                    id: state.messages.length,
                  },
                ],
              }));

              setIsWaiting(false);
            });
            setIsWaiting(true);
          }}
        />
      </Container>
    </ThemeProvider>
  );
}

export default App;
