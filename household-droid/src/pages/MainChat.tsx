import Chat from "../components/Chat";
import { useState } from "react";
import AgentSelection from "../components/AgentSelection";
import Output, { DialogEnum, type Message } from "../components/Output";
import { streamText } from "../services/api";
import { invokeAgent, type AgentSelections } from "../state/selectAgent";
import { getLoggedInJwt } from "../services/auth";
import {
  useNavigate,
  useNavigation,
  useOutletContext,
  useParams,
} from "react-router";
import CircularProgress from "@mui/material/CircularProgress";
import { useLoaderData, useRouteLoaderData } from "react-router";
const INIT_MESSAGE = {
  persona: DialogEnum.It,
  text: "Please start chatting!",
  id: 0,
};
interface MessagesAndText {
  messages: Message[];
  latestText: string;
}
const initMessageState = (messages: Message[]) => ({
  messages,
  latestText: "",
});

interface OutletContext {
  agentSelectionRef: React.Ref<HTMLDivElement>;
}

const MainChat = () => {
  const allParams = useParams();
  console.log(allParams);
  const { agent, sessionId } = allParams;
  //const {sessions} = useRouteLoaderData("sessionLoader");
  const { sessions, messages: historicalMessages } = useLoaderData();
  console.log(historicalMessages);
  console.log(sessions);
  const navigation = useNavigation();
  const navigate = useNavigate();
  const { agentSelectionRef } = useOutletContext() as OutletContext;
  const areMessagesInitialLoading = navigation.state === "loading"; // && navigation.location.pathname === "/";
  const [isWaiting, setIsWaiting] = useState(false);
  const [{ messages, latestText }, setMessages] = useState<MessagesAndText>(
    historicalMessages
      ? initMessageState(historicalMessages)
      : initMessageState([INIT_MESSAGE]),
  );

  const onStart = (value: string) => {
    setMessages((state) => ({
      latestText: state.latestText,
      messages: [
        ...state.messages,
        {
          role: DialogEnum.Me,
          content: value,
          id: state.messages.length,
        },
      ],
    }));
    setIsWaiting(true);
  };
  const onNew = (nextText: string) =>
    setMessages((state) => ({
      latestText: state.latestText + nextText,
      messages: state.messages,
    }));
  const onDone = () => {
    setMessages((state) => ({
      latestText: "",
      messages: [
        ...state.messages,
        {
          role: DialogEnum.It,
          content: state.latestText,
          id: state.messages.length,
        },
      ],
    }));
    setIsWaiting(false);
  };

  //have to do this janky workaround since useFetcher serializes responses and doesnt allow streaming
  const onSubmit = (selectedAgent: AgentSelections, value: string) => {
    const jwt = getLoggedInJwt();
    onStart(value);
    invokeAgent(selectedAgent, value, jwt as string, sessionId)
      .then(streamText(onNew, onDone))
      .catch(() => navigate("/login"));
  };

  return areMessagesInitialLoading ? (
    <CircularProgress />
  ) : (
    <>
      <AgentSelection
        ref={agentSelectionRef}
        selectedAgent={agent as AgentSelections}
      />
      <Output
        messages={messages}
        latestText={latestText}
        isWaiting={isWaiting}
      />
      <Chat onSubmit={onSubmit} selectedAgent={agent as AgentSelections} />
    </>
  );
};

export default MainChat;
