import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
//replace with multi-select
import Checkbox from "@mui/material/Checkbox";
import { Button } from "@mui/material";
import { useLoaderData } from "react-router";
const Settings = () => {
  const users = useLoaderData();
  console.log(users);
  return <>hello</>;
};
export default Settings;
/*<List
  sx={{ width: "100%", maxWidth: 360, bgcolor: "background.paper" }}
  aria-label="contacts"
>
  <ListItem
    disablePadding
    secondaryAction={
      <Checkbox
        edge="end"
        onChange={handleToggle(value)}
        checked={checked.includes(value)}
        inputProps={{ "aria-labelledby": labelId }}
      />
    }
  >
    <ListItemButton>
      <ListItemIcon>
        <StarIcon />
      </ListItemIcon>
      <ListItemText primary="Chelsea Otakan" />
    </ListItemButton>
  </ListItem>
</List>
<Button>Add User</Button> */
