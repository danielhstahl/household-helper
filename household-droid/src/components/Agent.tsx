import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import CardActions from "@mui/material/CardActions";
import Button from "@mui/material/Button";
import Typography from "@mui/material/Typography";
import Switch from "@mui/material/Switch";
import FormControlLabel from "@mui/material/FormControlLabel";
interface AgentProps {
  agentType: string;
  agentDescription: string;
  isDefault: boolean;
  setDefault: (_: boolean) => void;
}
const AgentSelection = ({
  agentType,
  agentDescription,
  isDefault,
  setDefault,
}: AgentProps) => {
  return (
    <Card sx={{ minWidth: 275 }}>
      <CardContent>
        <Typography variant="h5" component="div">
          {agentType}
        </Typography>
        <Typography variant="body2">{agentDescription}</Typography>
      </CardContent>
      <CardActions>
        <Button size="small">Use {agentType}</Button>
        <FormControlLabel
          control={
            <Switch
              checked={isDefault}
              onChange={(_, isChecked) => setDefault(isChecked)}
            />
          }
          label="Default agent"
        />
      </CardActions>
    </Card>
  );
};
export default AgentSelection;
