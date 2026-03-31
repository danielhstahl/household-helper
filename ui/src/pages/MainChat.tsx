import { useEffect, useState } from "react";
import Chat from "../components/Chat.tsx";
import AgentSelection from "../components/AgentSelection.tsx";
import Output from "../components/Output.tsx";
import { MessageTypeEnum, type Message } from "../services/models.tsx";
import { type AgentSelections } from "../state/selectAgent.tsx";
import { invokeAgent, type ChatToken } from "../services/api.tsx";
import { getLoggedInJwt } from "../state/localState.tsx";
import {
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
  reasoning: string;
}
const initMessageState = (messages: Message[]) => ({
  messages,
  latestText: "",
  reasoning: "",
});

interface OutletContext {
  agentSelectionRef: React.Ref<HTMLDivElement>;
}

const MainChat = () => {
  const { agent, sessionId } = useParams();
  const { sessions, messages: historicalMessages } = useLoaderData();
  const navigation = useNavigation();
  const [err, setError] = useState<string | null>(null);
  const { agentSelectionRef } = useOutletContext() as OutletContext;
  const areMessagesInitialLoading = navigation.state === "loading";
  const [isWaiting, setIsWaiting] = useState(false);

  const [{ messages, latestText, reasoning }, setMessages] =
    useState<MessagesAndText>(initMessageState(historicalMessages));
  //const [cot, setCot] = useState<string>("");
  useEffect(() => {
    setMessages((state) => ({
      latestText: state.latestText,
      reasoning: state.reasoning,
      messages: historicalMessages,
    }));
  }, [historicalMessages]);

  const onStart = (value: string) => {
    setMessages((state) => ({
      latestText: state.latestText,
      reasoning: state.reasoning,
      messages: [
        ...state.messages,
        {
          message_type: MessageTypeEnum.human,
          reasoning: "", //na for human messages
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
      //setCot((state) => state + nextText.tokens);
      setMessages((state) => ({
        latestText: state.latestText,
        reasoning: state.reasoning + nextText.tokens,
        messages: state.messages,
      }));
    } else {
      //setCot("");
      setMessages((state) => ({
        latestText: state.latestText + nextText.tokens,
        reasoning: state.reasoning,
        messages: state.messages,
      }));
    }
  };

  const completeMessageProcessing = () => {
    setMessages((state) => ({
      latestText: "",
      reasoning: "",
      messages: [
        ...state.messages,
        {
          message_type: MessageTypeEnum.ai,
          reasoning: state.reasoning,
          content: state.latestText,
          id: state.messages.length,
          timestamp: Date.now().toString(),
        },
      ],
    }));
  };
  const onDone = () => {
    setIsWaiting(false);
  };
  const onSubmit = (selectedAgent: AgentSelections, value: string) => {
    const jwt = getLoggedInJwt();
    onStart(value);
    setError(null);
    invokeAgent(selectedAgent, value, jwt as string, sessionId as string, onNew)
      .then(completeMessageProcessing)
      .catch(() => {
        setError(
          "Websocket error.  You either do not have permission to use this assistant or your session has expired.",
        );
      })
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
          latestCot={reasoning}
          latestText={latestText}
          isWaiting={isWaiting}
          err={err}
        />
        <Chat onSubmit={onSubmit} selectedAgent={agent as AgentSelections} />
      </Grid>
    </Grid>
  );
};

export default MainChat;
