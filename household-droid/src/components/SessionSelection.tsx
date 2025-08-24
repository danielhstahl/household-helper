import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemText from "@mui/material/ListItemText";
import DeleteIcon from "@mui/icons-material/Delete";
import IconButton from "@mui/material/IconButton";
import ListItemButton from "@mui/material/ListItemButton";
import Box from "@mui/material/Box";
import AddIcon from "@mui/icons-material/Add";
import Button from "@mui/material/Button";
import Typography from "@mui/material/Typography";
import { NavLink, Form, useLocation } from "react-router";
interface Session {
  id: string;
  session_start: string;
}
interface Props {
  sessions: Session[];
  selectedSessionId: string;
}
const trimDate = (datetime: string) => {
  const [dateHour, minutes] = datetime.split(":");
  return [dateHour, minutes].join(":");
};

const getNavLink = (location: string, sessionId: string) => {
  return location
    .split("/")
    .map((v, i, a) => (i !== a.length - 1 ? v : sessionId))
    .join("/");
};
const SessionSelection = ({ sessions, selectedSessionId }: Props) => {
  const location = useLocation();
  return (
    <Box style={{ paddingTop: 20 }}>
      <Typography
        sx={{ mt: 2, mb: 2, display: { xs: "none", sm: "block" } }}
        variant="h5"
        component="div"
      >
        Sessions
      </Typography>
      <Form noValidate autoComplete="off" method="post">
        <Button
          type="submit" // Crucial: triggers form submission
          variant="outlined"
          startIcon={<AddIcon />}
        >
          New Session
        </Button>
      </Form>
      <List sx={{ display: { xs: "none", sm: "block" } }}>
        {sessions.map(({ id, session_start }) => (
          <ListItem
            component={NavLink}
            to={getNavLink(location.pathname, id)}
            secondaryAction={
              <IconButton edge="end" aria-label="delete">
                <DeleteIcon />
              </IconButton>
            }
            disablePadding
            key={id}
            style={{ textDecoration: "none", color: "inherit" }}
          >
            <ListItemButton
              //role={undefined}
              selected={id === selectedSessionId}
            >
              <ListItemText primary={trimDate(session_start)} secondary={id} />
            </ListItemButton>
          </ListItem>
        ))}
      </List>
    </Box>
  );
};
export default SessionSelection;
