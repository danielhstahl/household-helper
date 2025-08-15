import Grid from "@mui/material/Grid";
import Paper from "@mui/material/Paper";
import Box from "@mui/material/Box";
import { useTheme } from "@mui/material";
import LinearProgress from "@mui/material/LinearProgress";
import ReactMarkdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import "katex/dist/katex.min.css";
import remarkGfm from "remark-gfm"; // For GitHub-flavored Markdown (tables, strikethrough, etc.)
import { memo } from "react";
export const DialogEnum = {
  Me: "me",
  It: "it",
} as const;
export type Dialog = (typeof DialogEnum)[keyof typeof DialogEnum];
export interface Message {
  persona: Dialog; //me or it
  text: string;
  id: number;
}
interface OutputProps {
  messages: Message[];
  isWaiting: boolean;
  latestText: string;
}
interface FormattedTextProps {
  text: string;
}
const FormattedText = memo(({ text }: FormattedTextProps) => (
  <ReactMarkdown
    remarkPlugins={[remarkGfm, remarkMath]}
    rehypePlugins={[rehypeKatex]}
  >
    {text}
  </ReactMarkdown>
));

const Output = ({ messages, isWaiting, latestText }: OutputProps) => {
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
          {messages.map(({ persona, text, id }) => (
            <Box
              key={id}
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
              <FormattedText text={text} />
            </Box>
          ))}
          {latestText !== "" && (
            <Box
              style={{
                alignSelf: "flex-start",
                maxWidth: "70%",
                borderRadius: 16,
                backgroundColor: theme.palette.grey[300],
                color: theme.palette.text.primary,
                padding: theme.spacing(1, 2),
                margin: theme.spacing(1),
                wordBreak: "break-word",
              }}
            >
              <FormattedText text={latestText} />
            </Box>
          )}
          {isWaiting && <LinearProgress />}
        </Paper>
      </Grid>
    </Grid>
  );
};

export default Output;
