import TextField from "@mui/material/TextField";
import Grid from "@mui/material/Grid";
import { type KeyboardEvent } from "react";
import { getAgentName, invokeAgent } from "../state/selectAgent";
import { useAgentParams } from "../state/AgentProvider";
import { streamText } from "../services/Api";
interface ChatProps {
  onNewText: (_: string) => void;
  onStart: (_: string) => void;
  onDone: () => void;
}
const Chat = ({ onStart, onNewText, onDone }: ChatProps) => {
  const { state: selectedAgent } = useAgentParams();
  const agentName = getAgentName(selectedAgent);
  const pressEnter = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key == "Enter") {
      //@ts-expect-error target.value exists in reality, even though it isn't on KeyboardEvent
      onStart(e.target.value);
      //@ts-expect-error target.value exists in reality, even though it isn't on KeyboardEvent
      invokeAgent(selectedAgent, e.target.value).then(
        streamText(onNewText, onDone),
      );
      e.preventDefault();
      //@ts-expect-error target.value exists in reality, even though it isn't on KeyboardEvent
      e.target.value = "";
    }
  };
  return (
    <Grid container spacing={2} style={{ paddingTop: 20, flexShrink: 0 }}>
      <Grid size={{ xs: 12 }}>
        <TextField
          label={`Chat or instruct your ${agentName}`}
          style={{ width: "100%" }}
          multiline
          rows={4}
          variant="filled"
          onKeyDown={pressEnter}
        />
      </Grid>
    </Grid>
  );
};

export default Chat;
