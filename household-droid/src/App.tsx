import "./App.css";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import Toolbar from "@mui/material/Toolbar";
import CssBaseline from "@mui/material/CssBaseline";
import Container from "@mui/material/Container";
import { Outlet } from "react-router";

const theme = createTheme({
  colorSchemes: {
    dark: true,
  },
});
function App() {
  return (
    <ThemeProvider theme={theme} defaultMode="light">
      <Container maxWidth={false} style={{ minHeight: "100%" }}>
        {/*Toolbar only here to push down below app bar*/}
        <Toolbar />
        <CssBaseline />
        <Outlet />
      </Container>
    </ThemeProvider>
  );
}

export default App;
