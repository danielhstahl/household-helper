import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import CardActions from "@mui/material/CardActions";
import CardActionArea from "@mui/material/CardActionArea";
import Button from "@mui/material/Button";
import Typography from "@mui/material/Typography";
import { useTheme } from "@mui/material";
//import Switch from "@mui/material/Switch";
//import FormControlLabel from "@mui/material/FormControlLabel";
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
          alignItems: "start",
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
        <CardContent style={{ verticalAlign: "top" }}>
          <Typography variant="h5" component="div">
            {agentType}
          </Typography>
          <Typography variant="body2">{agentDescription}</Typography>
        </CardContent>
      </CardActionArea>
    </Card>
  );
};
export default AgentSelection;
