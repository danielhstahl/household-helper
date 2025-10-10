import { useLoaderData, useNavigate } from "react-router";
import Grid from "@mui/material/Grid";
import Typography from "@mui/material/Typography";
import type { KnowledgeBase } from "../services/models";
import { styled } from "@mui/material/styles";
import CloudUpload from "@mui/icons-material/CloudUpload";
import Button from "@mui/material/Button";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import CardActions from "@mui/material/CardActions";
import { uploadFileToKnowledgeBase } from "../services/api";
import { getLoggedInJwt } from "../state/localState";
import { useState } from "react";
const VisuallyHiddenInput = styled("input")({
  clip: "rect(0 0 0 0)",
  clipPath: "inset(50%)",
  height: 1,
  overflow: "hidden",
  position: "absolute",
  bottom: 0,
  left: 0,
  whiteSpace: "nowrap",
  width: 1,
});
interface CardProps {
  kbId: number;
  kbName: string;
}
const KnowledgeBaseCard = ({ kbId, kbName }: CardProps) => {
  const navigate = useNavigate();
  const [loading, setIsLoading] = useState(false);
  const uploadFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files !== null) {
      const formData = new FormData();
      for (const file of files) {
        console.log(file);
        console.log(file instanceof File);
        formData.append("file", file);
      }
      const jwt = getLoggedInJwt();
      if (!jwt) {
        return navigate("/login");
      }
      setIsLoading(true);
      uploadFileToKnowledgeBase(kbId, formData, jwt).finally(() =>
        setIsLoading(false),
      );
    }
  };
  return (
    <Card variant="outlined">
      <CardContent>
        <Typography gutterBottom sx={{ color: "text.secondary", fontSize: 14 }}>
          Knowledge base {kbName}
        </Typography>
        <Typography variant="body2">
          Help give Draid knowledge! Upload text documents for ingestion into a
          Knowledge Base.
        </Typography>
      </CardContent>
      <CardActions>
        <Button
          size="small"
          component="label"
          role={undefined}
          variant="contained"
          tabIndex={-1}
          startIcon={<CloudUpload />}
          loading={loading}
        >
          Upload files
          <VisuallyHiddenInput type="file" onChange={uploadFile} multiple />
        </Button>
      </CardActions>
    </Card>
  );
};
const KnowledgeBaseUpload = () => {
  const knowledgeBases = useLoaderData() as KnowledgeBase[];
  return (
    <>
      {knowledgeBases.map(({ name, id }) => (
        <Grid size={{ xs: 12, md: 6 }} key={id}>
          <KnowledgeBaseCard kbName={name} kbId={id} />
        </Grid>
      ))}
    </>
  );
};
export default KnowledgeBaseUpload;
