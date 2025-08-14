import TextField from "@mui/material/TextField";
import Grid from "@mui/material/Grid";
import { type KeyboardEvent } from "react";
interface ChatProps {
  agentType: string;
  onEnter: (_: string) => void;
}
const Chat = ({ agentType, onEnter }: ChatProps) => {
  const pressEnter = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key == "Enter") {
      //@ts-expect-error target.value exists in reality, even though it isn't on KeyboardEvent
      onEnter(e.target.value);
      e.preventDefault();
      //@ts-expect-error target.value exists in reality, even though it isn't on KeyboardEvent
      e.target.value = "";
    }
  };
  return (
    <Grid container spacing={2} style={{ paddingTop: 20, flexShrink: 0 }}>
      <Grid size={{ xs: 12 }}>
        <TextField
          label={`Chat or instruct your ${agentType}`}
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
