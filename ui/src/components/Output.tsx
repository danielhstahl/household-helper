import Grid from "@mui/material/Grid";
import Paper from "@mui/material/Paper";
import Box from "@mui/material/Box";
import { IconButton, useTheme } from "@mui/material";
import LinearProgress from "@mui/material/LinearProgress";
import CircularProgress from "@mui/material/CircularProgress";
import Alert from "@mui/material/Alert";
import ReactMarkdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import rehypeHighlight from "rehype-highlight";
import "katex/dist/katex.min.css";
import remarkGfm from "remark-gfm"; // For GitHub-flavored Markdown (tables, strikethrough, etc.)
import { memo, useState } from "react";
import { MessageTypeEnum, type Message } from "../services/models";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import KeyboardArrowRightIcon from "@mui/icons-material/KeyboardArrowRight";
import { parseText } from "../services/utils";
interface OutputProps {
  messages: Message[];
  isWaiting: boolean;
  latestText: string;
  latestCot: string;
  loading: boolean;
  err: string | null;
}
interface FormattedTextProps {
  text: string;
  reasoning: string;
}

const FormattedText = memo(({ text, reasoning }: FormattedTextProps) => {
  const [showReasoning, setShowReasoning] = useState(false);
  const showButton = reasoning !== "" && text !== "";
  const deriveShowReasoning =
    showReasoning || (text === "" && reasoning !== "");
  return (
    <>
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkMath]}
        rehypePlugins={[rehypeKatex, [rehypeHighlight, { detect: false }]]}
      >
        {parseText(text)}
      </ReactMarkdown>
      {showButton && (
        <IconButton onClick={() => setShowReasoning((v) => !v)}>
          {showReasoning ? <ExpandMoreIcon /> : <KeyboardArrowRightIcon />}
        </IconButton>
      )}
      {deriveShowReasoning && (
        <div style={{ fontStyle: "italic" }}>
          <ReactMarkdown
            remarkPlugins={[remarkGfm, remarkMath]}
            rehypePlugins={[rehypeKatex, [rehypeHighlight, { detect: false }]]}
          >
            {parseText(reasoning)}
          </ReactMarkdown>
        </div>
      )}
    </>
  );
});

const Output = ({
  messages,
  isWaiting,
  latestText,
  latestCot,
  loading,
  err,
}: OutputProps) => {
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
          {messages.map(({ message_type, content, reasoning }, id) => (
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
                    : theme.palette.text.disabled,
                color:
                  message_type === MessageTypeEnum.human
                    ? theme.palette.primary.contrastText
                    : theme.palette.text.primary,
                padding: theme.spacing(1, 2),
                margin: theme.spacing(1),
                wordBreak: "break-word",
              }}
            >
              <FormattedText text={content} reasoning={reasoning} />
            </Box>
          ))}
          {latestCot !== "" && (
            <Box
              style={{
                alignSelf: "flex-start",
                maxWidth: "70%",
                borderRadius: 16,
                //fontStyle: "italic",
                backgroundColor: theme.palette.text.disabled,
                color: theme.palette.text.primary,
                padding: theme.spacing(1, 2),
                margin: theme.spacing(1),
                wordBreak: "break-word",
              }}
            >
              <FormattedText text="" reasoning={latestCot} />
            </Box>
          )}
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
              <FormattedText text={latestText} reasoning="" />
            </Box>
          )}
          {err && <Alert severity="error">{err}</Alert>}
          {isWaiting && <LinearProgress />}
        </Paper>
      </Grid>
    </Grid>
  );
};

export default Output;
