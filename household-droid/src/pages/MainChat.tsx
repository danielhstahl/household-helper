import { useEffect, useState } from "react";
import Chat from "../components/Chat.tsx";
import AgentSelection from "../components/AgentSelection.tsx";
import Output, { DialogEnum, type Message } from "../components/Output.tsx";
import { streamText } from "../services/api.tsx";
import { invokeAgent, type AgentSelections } from "../state/selectAgent.tsx";
import { getLoggedInJwt } from "../state/localState.tsx";
import {
  useNavigate,
  useNavigation,
  useOutletContext,
  useParams,
  useLoaderData,
} from "react-router";
import Grid from "@mui/material/Grid";
import SessionSelection from "../components/SessionSelection";

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
  const { agent, sessionId } = useParams();
  const { sessions, messages: historicalMessages } = useLoaderData();
  const navigation = useNavigation();
  const navigate = useNavigate();
  const { agentSelectionRef } = useOutletContext() as OutletContext;
  const areMessagesInitialLoading = navigation.state === "loading";
  const [isWaiting, setIsWaiting] = useState(false);
  const [{ messages, latestText }, setMessages] = useState<MessagesAndText>(
    initMessageState(historicalMessages),
  );
  useEffect(() => {
    setMessages((state) => ({
      latestText: state.latestText,
      messages: historicalMessages,
    }));
  }, [historicalMessages]);

  const onStart = (value: string) => {
    setMessages((state) => ({
      latestText: state.latestText,
      messages: [
        ...state.messages,
        {
          persona: DialogEnum.Me,
          content: value,
          id: state.messages.length,
          timestamp: Date.now().toString(),
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
          persona: DialogEnum.It,
          content: state.latestText,
          id: state.messages.length,
          timestamp: Date.now().toString(),
        },
      ],
    }));
    setIsWaiting(false);
  };

  //have to do this janky workaround since useFetcher serializes responses and doesnt allow streaming
  const onSubmit = (selectedAgent: AgentSelections, value: string) => {
    const jwt = getLoggedInJwt();
    onStart(value);
    return invokeAgent(selectedAgent, value, jwt as string, sessionId as string)
      .then(streamText(onNew, onDone))
      .catch(() => navigate("/login"));
  };

  return (
    <Grid container spacing={2}>
      <Grid size={{ xs: 12, md: 3 }}>
        <SessionSelection
          sessions={sessions}
          selectedSessionId={sessionId as string}
        />
      </Grid>
      <Grid size={{ xs: 12, md: 9 }}>
        <AgentSelection
          ref={agentSelectionRef}
          selectedAgent={agent as AgentSelections}
          selectedSessionId={sessionId as string}
        />
        <Output
          loading={areMessagesInitialLoading}
          messages={messages}
          latestText={latestText}
          isWaiting={isWaiting}
        />
        <Chat onSubmit={onSubmit} selectedAgent={agent as AgentSelections} />
      </Grid>
    </Grid>
  );
};

export default MainChat;
