import "./App.css";
import { useState, useRef, useEffect } from "react";
import AppBarDroid from "./components/AppBar";
import AgentSelection from "./components/AgentSelection";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import Toolbar from "@mui/material/Toolbar";
import CssBaseline from "@mui/material/CssBaseline";
import Container from "@mui/material/Container";
import Chat from "./components/Chat";
import Output, { DialogEnum, type Message } from "./components/Output";
import { AgentProvider } from "./state/AgentProvider";
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
const theme = createTheme({
  colorSchemes: {
    dark: true,
  },
});
function App() {
  //const theme = createTheme();

  const [{ messages, latestText }, setMessages] =
    useState<MessagesAndText>(initMessageAndText);
  const [isWaiting, setIsWaiting] = useState(false);
  const agentSelectionRef = useRef(null);
  const [agentSelectionHeight, setAgentSelectionHeight] = useState(0);
  useEffect(() => {
    if (agentSelectionRef.current) {
      //@ts-expect-error need to start with null, but offsetHeight exists
      setAgentSelectionHeight(agentSelectionRef.current.offsetHeight);
    }
  }, []);
  return (
    <ThemeProvider theme={theme} defaultMode="light">
      <Container maxWidth={false} style={{ minHeight: "100%" }}>
        {/*Toolbar only here to push down below app bar*/}
        <Toolbar />
        <CssBaseline />
        <AgentProvider>
          <AppBarDroid threshold={agentSelectionHeight} />
          <AgentSelection ref={agentSelectionRef} />
          <Output
            messages={messages}
            latestText={latestText}
            isWaiting={isWaiting}
          />
          <Chat
            onStart={(v: string) => {
              setMessages((state) => ({
                latestText: state.latestText,
                messages: [
                  ...state.messages,
                  {
                    persona: DialogEnum.Me,
                    text: v,
                    id: state.messages.length,
                  },
                ],
              }));
              setIsWaiting(true);
            }}
            onNewText={(v: string) => {
              setMessages((state) => ({
                latestText: state.latestText + v,
                messages: state.messages,
              }));
            }}
            onDone={() => {
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
            }}
          />
        </AgentProvider>
      </Container>
    </ThemeProvider>
  );
}

export default App;
