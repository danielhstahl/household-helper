import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import CardActionArea from "@mui/material/CardActionArea";
import Typography from "@mui/material/Typography";
import { useTheme } from "@mui/material";
interface AgentProps {
  agentType: string;
  agentDescription: string;
  isDefault: boolean;
  setDefault: () => void;
}
const AgentSelection = ({
  agentType,
  agentDescription,
  isDefault,
  setDefault,
}: AgentProps) => {
  const theme = useTheme();
  return (
    <Card sx={{ minWidth: 275, height: "100%" }} variant="outlined">
      <CardActionArea
        onClick={() => setDefault()}
        data-active={isDefault ? "" : undefined}
        sx={{
          //alignItems: "start",
          height: "100%",
          "&[data-active]": {
            border: `1px solid`,
            //backgroundColor: "action.selected",
            borderColor: theme.palette.primary.main,
            borderyStyle: "solid",
            "&:hover": {
              backgroundColor: "action.selectedHover",
            },
          },
        }}
      >
        <CardContent style={{ height: "100%" }}>
          <Typography gutterBottom variant="h5" component="div">
            {agentType}
          </Typography>
          <Typography variant="body2">{agentDescription}</Typography>
        </CardContent>
      </CardActionArea>
    </Card>
  );
};
export default AgentSelection;
