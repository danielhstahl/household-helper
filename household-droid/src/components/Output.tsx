import Grid from "@mui/material/Grid";
import Typography from "@mui/material/Typography";
import Paper from "@mui/material/Paper";
import Chip from "@mui/material/Chip";
import Box from "@mui/material/Box";
import { useTheme } from "@mui/material";
const DialogEnum = {
  Me: "me",
  It: "it",
} as const;
type Dialog = (typeof DialogEnum)[keyof typeof DialogEnum];
interface Message {
  persona: Dialog; //me or it
  text: string;
}
interface OutputProps {
  agentType: string;
  messages: Message[];
}

const Output = ({ agentType, messages }: OutputProps) => {
  const theme = useTheme();
  return (
    <Grid
      container
      spacing={2}
      style={{
        paddingTop: 20,
        minHeight: 0,
      }}
    >
      <Grid size={{ xs: 12 }}>
        <Paper
          style={{
            display: "flex",
            flexDirection: "column",
            maxHeight: 600,
            overflow: "auto",
          }}
        >
          {messages.map(({ persona, text }) => (
            <Box
              style={{
                alignSelf:
                  persona === DialogEnum.Me ? "flex-end" : "flex-start",
                maxWidth: "70%",
                borderRadius: 16,
                backgroundColor:
                  persona === DialogEnum.Me
                    ? theme.palette.primary.main
                    : theme.palette.grey[300],
                color:
                  persona === DialogEnum.Me
                    ? theme.palette.primary.contrastText
                    : theme.palette.text.primary,
                padding: theme.spacing(1, 2),
                margin: theme.spacing(1),
                wordBreak: "break-word",
              }}
            >
              {text}
            </Box>
          ))}
        </Paper>
      </Grid>
    </Grid>
  );
};

export default Output;
