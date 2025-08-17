import AppBarDroid from "../components/AppBar";
import Chat from "../components/Chat";
import { useState, useRef, useEffect } from "react";
import AgentSelection from "../components/AgentSelection";
import Output, { DialogEnum, type Message } from "../components/Output";
import { AgentProvider } from "../state/AgentProvider";
import { useLoaderData } from "react-router";

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
const MainPage = () => {
  const { jwt, session } = useLoaderData();
  const { sessions, user } = session;
  console.log(sessions);
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
    <AgentProvider>
      <AppBarDroid
        threshold={agentSelectionHeight}
        isAdmin={user.roles.find((v: string) => v === "admin") ? true : false}
      />
      <AgentSelection ref={agentSelectionRef} />
      <Output
        messages={messages}
        latestText={latestText}
        isWaiting={isWaiting}
      />
      <Chat
        jwt={jwt}
        sessionId={undefined}
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
  );
};

export default MainPage;
