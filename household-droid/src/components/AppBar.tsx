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
import { useAgentParams } from "../state/AgentProvider";
import {
  AgentSelectionsEnum,
  getAgentName,
  type AgentSelections,
} from "../state/selectAgent";
import { useTheme } from "@mui/material";
import useScrollTrigger from "@mui/material/useScrollTrigger";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useColorScheme } from "@mui/material/styles";
import { Link as RouterLink } from "react-router";
const AppBarDroid = ({
  threshold,
  isAdmin,
}: {
  threshold: number;
  isAdmin: boolean;
}) => {
  const { state: selectedAgent, dispatch: setSelectedAgent } = useAgentParams();
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
            value={selectedAgent}
            onChange={(event) => {
              setSelectedAgent(event.target.value);
            }}
            variant="standard"
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
            <MenuItem value={AgentSelectionsEnum.HELPER_INDEX}>
              {getAgentName(AgentSelectionsEnum.HELPER_INDEX)}
            </MenuItem>
            <MenuItem value={AgentSelectionsEnum.TUTOR_INDEX}>
              {getAgentName(AgentSelectionsEnum.TUTOR_INDEX)}
            </MenuItem>
          </Select>
        )}
        {!isLargerThanXS && (
          <NativeSelect
            id="menu-appbar"
            value={selectedAgent}
            onChange={(event) => {
              setSelectedAgent(parseInt(event.target.value) as AgentSelections);
            }}
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
            <option value={AgentSelectionsEnum.HELPER_INDEX}>
              {getAgentName(AgentSelectionsEnum.HELPER_INDEX)}
            </option>
            <option value={AgentSelectionsEnum.TUTOR_INDEX}>
              {getAgentName(AgentSelectionsEnum.TUTOR_INDEX)}
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
            component={RouterLink}
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
