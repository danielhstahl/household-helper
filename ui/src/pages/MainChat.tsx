import { useEffect, useState } from "react";
import Chat from "../components/Chat.tsx";
import AgentSelection from "../components/AgentSelection.tsx";
import Output from "../components/Output.tsx";
import { MessageTypeEnum, type Message } from "../services/models.tsx";
import { type AgentSelections } from "../state/selectAgent.tsx";
import { invokeAgent } from "../services/api.tsx";
import { getLoggedInJwt } from "../state/localState.tsx";
import {
  useNavigation,
  useOutletContext,
  useParams,
  useLoaderData,
} from "react-router";
import Grid from "@mui/material/Grid";
import SessionSelection from "../components/SessionSelection";
import { useNavigate } from "react-router";

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

interface ChatToken {
  tokenType: string;
  tokens: string;
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
  const [cot, setCot] = useState<string>("");
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
          message_type: MessageTypeEnum.human,
          content: value,
          id: state.messages.length,
          timestamp: Date.now().toString(),
        },
      ],
    }));
    setIsWaiting(true);
  };
  const onNew = (nextText: ChatToken) => {
    if (nextText.tokenType === "ChainOfThought") {
      setCot((state) => state + nextText.tokens);
    } else {
      setCot("");
      setMessages((state) => ({
        latestText: state.latestText + nextText.tokens,
        messages: state.messages,
      }));
    }
  };
  const onDone = () => {
    setMessages((state) => ({
      latestText: "",
      messages: [
        ...state.messages,
        {
          message_type: MessageTypeEnum.ai,
          content: state.latestText,
          id: state.messages.length,
          timestamp: Date.now().toString(),
        },
      ],
    }));
    setIsWaiting(false);
  };
  const onSubmit = (selectedAgent: AgentSelections, value: string) => {
    const jwt = getLoggedInJwt();
    onStart(value);
    invokeAgent(selectedAgent, value, jwt as string, sessionId as string, onNew)
      .catch(() => navigate("/login"))
      .finally(onDone);
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
          latestCot={cot}
          latestText={latestText}
          isWaiting={isWaiting}
        />
        <Chat onSubmit={onSubmit} selectedAgent={agent as AgentSelections} />
      </Grid>
    </Grid>
  );
};

export default MainChat;
