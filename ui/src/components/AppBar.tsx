import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import MenuItem from "@mui/material/MenuItem";
import Select, { type SelectChangeEvent } from "@mui/material/Select";
import LightMode from "@mui/icons-material/LightMode";
import DarkMode from "@mui/icons-material/DarkMode";
import IconButton from "@mui/material/IconButton";
import SettingsIcon from "@mui/icons-material/Settings";
import LogoutIcon from "@mui/icons-material/Logout";
import {
  AgentSelectionsEnum,
  getAgentName,
  type AgentSelections,
} from "../state/selectAgent.tsx";
import { useTheme } from "@mui/material";
import useScrollTrigger from "@mui/material/useScrollTrigger";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useColorScheme } from "@mui/material/styles";
import { NavLink, useNavigate } from "react-router";
import { useEffect } from "react";

const LIGHT_THEME_URL =
  "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github.min.css";
const DARK_THEME_URL =
  "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github-dark.min.css";
const AppBarDroid = ({
  threshold,
  isAdmin,
  agent,
  sessionId,
}: {
  threshold: number;
  isAdmin: boolean;
  agent: string;
  sessionId: string;
}) => {
  const theme = useTheme();
  const navigate = useNavigate();
  const handleSelectChange = (event: SelectChangeEvent<string>) => {
    const agent = event.target.value;
    navigate(`/${agent}/${sessionId}`);
  };
  const trigger = useScrollTrigger({ threshold, disableHysteresis: true });
  const isLargerThanSm = useMediaQuery(theme.breakpoints.up("md"));
  const isLargerThanXS = useMediaQuery(theme.breakpoints.up("sm"));
  const { mode, setMode } = useColorScheme();
  //hacky, but effective.  load from CDN on mode change.
  useEffect(() => {
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = mode === "light" ? LIGHT_THEME_URL : DARK_THEME_URL;
    document.head.appendChild(link);
    return () => {
      document.head.removeChild(link);
    };
  }, [mode]);
  return (
    <AppBar>
      <Toolbar>
        <Typography
          variant="h6"
          component={NavLink}
          to="/"
          sx={{ flexGrow: 1 }}
          style={{ textDecoration: "none", color: "inherit" }}
        >
          Draid
        </Typography>
        {!trigger && isLargerThanSm && (
          <Typography component="div">
            This is the droid you've been looking for!
          </Typography>
        )}

        {(trigger || !isLargerThanXS) && agent && (
          <Select
            id="menu-appbar"
            value={agent as AgentSelections}
            variant="standard"
            renderValue={getAgentName}
            sx={{
              borderRadius: theme.shape.borderRadius,
              "& .MuiSelect-select": {
                color: theme.palette.common.white,
              },
              "& .MuiSvgIcon-root": {
                color: theme.palette.common.white,
              },
            }}
            onChange={handleSelectChange}
          >
            <MenuItem value={AgentSelectionsEnum.HELPER}>
              {getAgentName(AgentSelectionsEnum.HELPER)}
            </MenuItem>
            <MenuItem value={AgentSelectionsEnum.TUTOR}>
              {getAgentName(AgentSelectionsEnum.TUTOR)}
            </MenuItem>
          </Select>
        )}
        <IconButton
          aria-label="switch-mode"
          color="inherit"
          onClick={() => setMode(mode === "light" ? "dark" : "light")}
        >
          {mode === "light" ? <DarkMode /> : <LightMode />}
        </IconButton>
        {isAdmin && (
          <IconButton
            aria-label="settings"
            color="inherit"
            component={NavLink}
            to="/settings"
          >
            <SettingsIcon />
          </IconButton>
        )}
        <IconButton
          aria-label="logout"
          color="inherit"
          component={NavLink}
          to="/logout"
        >
          <LogoutIcon />
        </IconButton>
      </Toolbar>
    </AppBar>
  );
};

export default AppBarDroid;
