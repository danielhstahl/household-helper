import AppBarDroid from "../components/AppBar";
import { useState, useRef, useEffect } from "react";
import { AgentProvider } from "../state/AgentProvider";
import { useLoaderData } from "react-router";
import { Outlet } from "react-router";

import Select from "@mui/material/Select";
import MenuItem from "@mui/material/MenuItem";
import { NavLink } from "react-router";

interface User {
  roles: string[];
  id: string;
  username: string;
}
/*<Select
  //value={selectedSession}
  label="Select Session"
  //onChange={(e) => setSelectedSession(e.target.value)}
>
  {sessions.map((v, i) => (
    <MenuItem component={NavLink} to="/" key={i} value={v}>
      {v}
    </MenuItem>
  ))}
</Select> */

const MainPage = () => {
  const user = useLoaderData<User>();
  //const { sessions, user } = session;

  const agentSelectionRef = useRef(null);
  const [agentSelectionHeight, setAgentSelectionHeight] = useState(0);
  //const [selectedSession, setSelectedSession] = useState<string>("");
  useEffect(() => {
    if (agentSelectionRef.current) {
      //@ts-expect-error need to start with null, but offsetHeight exists
      setAgentSelectionHeight(agentSelectionRef.current.offsetHeight);
    }
  }, []);
  return (
    <AgentProvider>
      <AppBarDroid
        threshold={agentSelectionHeight}
        isAdmin={user.roles.find((v: string) => v === "admin") ? true : false} //needs session to have been called
      />

      <Outlet context={{ agentSelectionRef }} />
    </AgentProvider>
  );
};

export default MainPage;
