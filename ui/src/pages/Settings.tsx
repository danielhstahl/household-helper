import Grid from "@mui/material/Grid";
import { Outlet } from "react-router";
import Tabs from "@mui/material/Tabs";
import Tab from "@mui/material/Tab";
import { NavLink, useLocation, useNavigation } from "react-router";
import CircularProgress from "@mui/material/CircularProgress";

function a11yProps(index: number) {
  return {
    id: `simple-tab-${index}`,
    "aria-controls": `simple-tabpanel-${index}`,
  };
}

const Settings = () => {
  const location = useLocation();
  const [path] = location.pathname.split("/").reverse();
  const navigation = useNavigation();

  return (
    <>
      <Tabs value={path} aria-label="Settings">
        <Tab
          component={NavLink}
          to="users"
          value="users"
          label="Users"
          {...a11yProps(0)}
        />
        <Tab
          component={NavLink}
          to="metrics"
          value="metrics"
          label="Metrics"
          {...a11yProps(1)}
        />
        <Tab
          component={NavLink}
          to="knowledgebase"
          value="knowledgebase"
          label="Knowledge Base"
          {...a11yProps(2)}
        />
      </Tabs>
      {navigation.state === "loading" && <CircularProgress />}
      <Grid container spacing={2} style={{ paddingTop: 20 }}>
        <Outlet />
      </Grid>
    </>
  );
};
export default Settings;
