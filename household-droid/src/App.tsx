import "./App.css";
import AppBarDroid from "./components/AppBar";
import AgentSelection from "./components/AgentSelection";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import Box from "@mui/material/Box";
import Toolbar from "@mui/material/Toolbar";
import CssBaseline from "@mui/material/CssBaseline";
import Container from "@mui/material/Container";
import Chat from "./components/Chat";
import Output from "./components/Output";
function App() {
  const theme = createTheme();
  return (
    <ThemeProvider theme={theme}>
      <Container maxWidth={false} style={{ minHeight: "100%" }}>
        {/*Toolbar only here to push down below app bar*/}
        <Toolbar />
        <CssBaseline />
        <AppBarDroid />
        <AgentSelection />
        <Output
          messages={[
            { persona: "me", text: "hello world" },
            { persona: "it", text: "why yes, hello" },
            { persona: "it", text: "why yes, hello" },
            { persona: "it", text: "why yes, hello" },
          ]}
          agentType="helper"
        />
        <Chat
          agentType="helper"
          onEnter={(v) => {
            console.log(v);
          }}
        />
      </Container>
    </ThemeProvider>
  );
}

export default App;
