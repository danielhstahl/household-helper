import AppBarDroid from "../components/AppBar.tsx";
import { useState, useRef, useEffect } from "react";
import { useLoaderData, Outlet, useParams } from "react-router";
import { type UserResponse, RoleTypeEnum } from "../services/models.tsx";

const MainPage = () => {
  const user = useLoaderData<UserResponse>();
  const { agent, sessionId } = useParams();
  const agentSelectionRef = useRef(null);
  const [agentSelectionHeight, setAgentSelectionHeight] = useState(0);
  useEffect(() => {
    if (agentSelectionRef.current) {
      //@ts-expect-error need to start with null, but offsetHeight exists
      setAgentSelectionHeight(agentSelectionRef.current.offsetHeight);
    }
  }, []);
  return (
    <>
      <AppBarDroid
        threshold={agentSelectionHeight}
        isAdmin={
          user.roles.find((v: string) => v === RoleTypeEnum.admin)
            ? true
            : false
        }
        agent={agent || ""}
        sessionId={sessionId || ""}
      />
      <Outlet context={{ agentSelectionRef }} />
    </>
  );
};

export default MainPage;
