import TextField from "@mui/material/TextField";
import Grid from "@mui/material/Grid";
import { type KeyboardEvent } from "react";
import { getAgentName, type AgentSelections } from "../state/selectAgent.tsx";

interface ChatProps {
  onSubmit: (agent: AgentSelections, _: string) => void;
  selectedAgent: AgentSelections;
}
const Chat = ({ onSubmit, selectedAgent }: ChatProps) => {
  const agentName = getAgentName(selectedAgent);
  const pressEnter = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key == "Enter" && !e.shiftKey) {
      e.preventDefault();
      const target = e.target as HTMLInputElement;
      onSubmit(selectedAgent, target.value);
      target.value = "";
    }
  };
  return (
    <Grid container spacing={2} style={{ paddingTop: 20, flexShrink: 0 }}>
      <Grid size={{ xs: 12 }}>
        <TextField
          autoFocus={true}
          label={`Chat or instruct your ${agentName}`}
          style={{ width: "100%" }}
          name="chat"
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
