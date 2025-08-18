//import List from "@mui/material/List";
//import ListItem from "@mui/material/ListItem";
//replace with multi-select
import Button from "@mui/material/Button";
import { type Theme, useTheme } from "@mui/material";
import OutlinedInput from "@mui/material/OutlinedInput";
import Chip from "@mui/material/Chip";
import Box from "@mui/material/Box";
import MenuItem from "@mui/material/MenuItem";
import { useLoaderData } from "react-router";
import Select, { type SelectChangeEvent } from "@mui/material/Select";
//import ListItemButton from "@mui/material/ListItemButton";
//import ListItemText from "@mui/material/ListItemText";
//import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Paper from "@mui/material/Paper";
import Table from "../components/TableX";

import { useState } from "react";
interface User {
  username: string;
  roles: string[];
}
const all_roles = [
  {
    value: "admin",
    label: "Admin",
  },
  {
    value: "tutor",
    label: "Tutor",
  },
  {
    value: "helper",
    label: "Helper",
  },
];
function getStyles(name: string, personName: readonly string[], theme: Theme) {
  return {
    fontWeight: personName.includes(name)
      ? theme.typography.fontWeightMedium
      : theme.typography.fontWeightRegular,
  };
}
const ITEM_HEIGHT = 48;
const ITEM_PADDING_TOP = 8;
const MenuProps = {
  PaperProps: {
    style: {
      maxHeight: ITEM_HEIGHT * 4.5 + ITEM_PADDING_TOP,
      width: 250,
    },
  },
};
const Settings = () => {
  const users = useLoaderData() as User[];
  const usersWithId = users.map((user, index) => ({ ...user, id: index }));
  const theme = useTheme();
  const [localUsers, setLocalUsers] = useState<User[]>(users);
  const handleChange = (
    event: SelectChangeEvent<string[]>,
    selectedUsername: string,
  ) => {
    const {
      target: { value },
    } = event;
    const newRoles = typeof value === "string" ? value.split(",") : value;
    setLocalUsers((localUsers) =>
      localUsers.map(({ username, roles }) => {
        return username === selectedUsername
          ? { username, roles: newRoles }
          : { username, roles };
      }),
    );
  };
  const handleNewUser = () => {
    setLocalUsers((localUsers) => [
      ...localUsers,
      { username: `new user ${localUsers.length}`, roles: [] },
    ]);
  };
  return <Table users={usersWithId} />;
  /*<>
  <TableContainer component={Paper}>
    <Table sx={{ minWidth: 650 }} aria-label="caption for new user">
      <caption>
        <Button onClick={handleNewUser}>Add User</Button>
      </caption>
      <TableHead>
        <TableRow>
          <TableCell>Username</TableCell>
          <TableCell align="left">Password</TableCell>
          <TableCell align="left">Roles</TableCell>
          <TableCell align="left"></TableCell>
        </TableRow>
      </TableHead>
      <TableBody>
        {localUsers.map(({ username, roles }) => (
          <TableRow key={username}>
            <TableCell component="th" scope="row">
              {username}
            </TableCell>
            <TableCell component="th" align="left">
              <Button>Regenerate Password</Button>
            </TableCell>
            <TableCell align="left">
              <Select
                labelId="demo-multiple-chip-label"
                id="demo-multiple-chip"
                multiple
                value={roles}
                onChange={(event) => handleChange(event, username)}
                input={
                  <OutlinedInput id="select-multiple-chip" label="Roles" />
                }
                renderValue={(selected) => (
                  <Box sx={{ display: "flex", flexWrap: "wrap", gap: 0.5 }}>
                    {selected.map((value) => (
                      <Chip key={value} label={value} />
                    ))}
                  </Box>
                )}
                MenuProps={MenuProps}
              >
                {all_roles.map(({ value, label }) => (
                  <MenuItem
                    key={value}
                    value={value}
                    style={getStyles(value, roles, theme)}
                  >
                    {label}
                  </MenuItem>
                ))}
              </Select>
            </TableCell>
            <TableCell component="th" align="left">
              <Button>Remove</Button>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  </TableContainer>
</> */
  /*(
    <>
      <List
        sx={{ width: "100%", maxWidth: 360, bgcolor: "background.paper" }}
        aria-label="contacts"
      >
        {users.map(({ username, roles }) => (
          <ListItem
            key={username}
            disablePadding
            secondaryAction={
              <Select
                labelId="demo-multiple-chip-label"
                id="demo-multiple-chip"
                multiple
                value={roles}
                //onChange={handleChange}
                input={
                  <OutlinedInput id="select-multiple-chip" label="Roles" />
                }
                renderValue={(selected) => (
                  <Box sx={{ display: "flex", flexWrap: "wrap", gap: 0.5 }}>
                    {selected.map((value) => (
                      <Chip key={value} label={value} />
                    ))}
                  </Box>
                )}
                MenuProps={MenuProps}
              >
                {all_roles.map(({ value, label }) => (
                  <MenuItem
                    key={value}
                    value={value}
                    style={getStyles(value, roles, theme)}
                  >
                    {label}
                  </MenuItem>
                ))}
              </Select>
            }
          >
            <ListItemButton>
              <ListItemText primary={username} />
            </ListItemButton>
          </ListItem>
        ))}
      </List>
      <Button>Add User</Button>
    </>
  );*/
};
export default Settings;
/**/
