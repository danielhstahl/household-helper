import Grid from "@mui/material/Grid";
import Paper from "@mui/material/Paper";
import Box from "@mui/material/Box";
import { useTheme } from "@mui/material";
import LinearProgress from "@mui/material/LinearProgress";
import CircularProgress from "@mui/material/CircularProgress";
import ReactMarkdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import "katex/dist/katex.min.css";
import remarkGfm from "remark-gfm"; // For GitHub-flavored Markdown (tables, strikethrough, etc.)
import { memo } from "react";
import { MessageTypeEnum, type Message } from "../services/models";

interface OutputProps {
  messages: Message[];
  isWaiting: boolean;
  latestText: string;
  loading: boolean;
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

const Output = ({ messages, isWaiting, latestText, loading }: OutputProps) => {
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
          }}
        >
          {loading && <CircularProgress />}
          {messages.map(({ message_type, content }, id) => (
            <Box
              key={id}
              style={{
                alignSelf:
                  message_type === MessageTypeEnum.human
                    ? "flex-end"
                    : "flex-start",
                maxWidth: "70%",
                borderRadius: 16,
                backgroundColor:
                  message_type === MessageTypeEnum.human
                    ? theme.palette.primary.main
                    : theme.palette.text.disabled, //theme.palette.grey[300],
                color:
                  message_type === MessageTypeEnum.human
                    ? theme.palette.primary.contrastText
                    : theme.palette.text.primary,
                padding: theme.spacing(1, 2),
                margin: theme.spacing(1),
                wordBreak: "break-word",
              }}
            >
              <FormattedText text={content} />
            </Box>
          ))}
          {latestText !== "" && (
            <Box
              style={{
                alignSelf: "flex-start",
                maxWidth: "70%",
                borderRadius: 16,
                backgroundColor: theme.palette.text.disabled,
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
