import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import MenuItem from "@mui/material/MenuItem";
import Select from "@mui/material/Select";
import LightMode from "@mui/icons-material/LightMode";
import DarkMode from "@mui/icons-material/DarkMode";
import IconButton from "@mui/material/IconButton";
import NativeSelect from "@mui/material/NativeSelect";
import SettingsIcon from "@mui/icons-material/Settings";
import { useParams } from "react-router";
import {
  AgentSelectionsEnum,
  getAgentName,
  type AgentSelections,
} from "../state/selectAgent";
import { useTheme } from "@mui/material";
import useScrollTrigger from "@mui/material/useScrollTrigger";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useColorScheme } from "@mui/material/styles";
import { NavLink } from "react-router";

const AppBarDroid = ({
  threshold,
  isAdmin,
}: {
  threshold: number;
  isAdmin: boolean;
}) => {
  const { agent, sessionId } = useParams();
  const theme = useTheme();
  const trigger = useScrollTrigger({ threshold, disableHysteresis: true });
  const isLargerThanXS = useMediaQuery(theme.breakpoints.up("sm"));
  const { mode, setMode } = useColorScheme();
  return (
    <AppBar>
      <Toolbar>
        <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
          Household Droid
        </Typography>
        {!trigger && isLargerThanXS && (
          <Typography component="div">
            This is the droid you've been looking for!
          </Typography>
        )}

        {trigger && isLargerThanXS && (
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
          >
            <NavLink
              to={`/${AgentSelectionsEnum.HELPER}/${sessionId}`}
              style={{ textDecoration: "none", color: "inherit" }}
            >
              <MenuItem value={AgentSelectionsEnum.HELPER}>
                {getAgentName(AgentSelectionsEnum.HELPER)}
              </MenuItem>
            </NavLink>
            <NavLink
              to={`/${AgentSelectionsEnum.TUTOR}/${sessionId}`}
              style={{ textDecoration: "none", color: "inherit" }}
            >
              <MenuItem value={AgentSelectionsEnum.TUTOR}>
                {getAgentName(AgentSelectionsEnum.TUTOR)}
              </MenuItem>
            </NavLink>
          </Select>
        )}
        {!isLargerThanXS && (
          <NativeSelect
            id="menu-appbar"
            value={agent}
            variant="standard"
            sx={{
              borderRadius: theme.shape.borderRadius,
              "& .MuiNativeSelect-select": {
                color: theme.palette.common.white,
              },
              "& .MuiSvgIcon-root": {
                color: theme.palette.common.white,
              },
            }}
          >
            <option value={AgentSelectionsEnum.HELPER}>
              {getAgentName(AgentSelectionsEnum.HELPER)}
            </option>
            <option value={AgentSelectionsEnum.TUTOR}>
              {getAgentName(AgentSelectionsEnum.TUTOR)}
            </option>
          </NativeSelect>
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
      </Toolbar>
    </AppBar>
  );
};

export default AppBarDroid;
