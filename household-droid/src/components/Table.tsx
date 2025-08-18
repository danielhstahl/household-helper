import { useState } from "react";
import { alpha } from "@mui/material/styles";

import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableContainer from "@mui/material/TableContainer";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import Paper from "@mui/material/Paper";
import Checkbox from "@mui/material/Checkbox";
import IconButton from "@mui/material/IconButton";
import Tooltip from "@mui/material/Tooltip";
import DeleteIcon from "@mui/icons-material/Delete";
import Button from "@mui/material/Button";
import OutlinedInput from "@mui/material/OutlinedInput";
import Chip from "@mui/material/Chip";
import Box from "@mui/material/Box";
import MenuItem from "@mui/material/MenuItem";
import Select, { type SelectChangeEvent } from "@mui/material/Select";
import TextField from "@mui/material/TextField";
import { type Theme, useTheme } from "@mui/material";
interface EnhancedTableProps {
  numSelected: number;
  onSelectAllClick: (event: React.ChangeEvent<HTMLInputElement>) => void;
  rowCount: number;
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

function getStyles(name: string, personName: readonly string[], theme: Theme) {
  return {
    fontWeight: personName.includes(name)
      ? theme.typography.fontWeightMedium
      : theme.typography.fontWeightRegular,
  };
}

function EnhancedTableHead(props: EnhancedTableProps) {
  const { onSelectAllClick, numSelected, rowCount } = props;
  return (
    <TableHead>
      <TableRow>
        <TableCell padding="checkbox">
          <Checkbox
            color="primary"
            indeterminate={numSelected > 0 && numSelected < rowCount}
            checked={rowCount > 0 && numSelected === rowCount}
            onChange={onSelectAllClick}
            slotProps={{
              input: {
                "aria-label": "select all desserts",
              },
            }}
          />
        </TableCell>
        <TableCell>Username</TableCell>
        <TableCell align="left">Password</TableCell>
        <TableCell align="left">Roles</TableCell>
      </TableRow>
    </TableHead>
  );
}
interface EnhancedTableToolbarProps {
  numSelected: number;
  onDelete: () => void;
}
function EnhancedTableToolbar(props: EnhancedTableToolbarProps) {
  const { numSelected, onDelete } = props;
  return (
    <Toolbar
      sx={[
        {
          pl: { sm: 2 },
          pr: { xs: 1, sm: 1 },
        },
        numSelected > 0 && {
          bgcolor: (theme) =>
            alpha(
              theme.palette.primary.main,
              theme.palette.action.activatedOpacity,
            ),
        },
      ]}
    >
      {numSelected > 0 ? (
        <Typography
          sx={{ flex: "1 1 100%" }}
          color="inherit"
          variant="subtitle1"
          component="div"
        >
          {numSelected} selected
        </Typography>
      ) : (
        <Typography
          sx={{ flex: "1 1 100%" }}
          variant="h6"
          id="tableTitle"
          component="div"
        >
          Users
        </Typography>
      )}
      {numSelected > 0 && (
        <Tooltip title="Delete">
          <IconButton onClick={onDelete}>
            <DeleteIcon />
          </IconButton>
        </Tooltip>
      )}
    </Toolbar>
  );
}
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
export default function EnhancedTable({ users }: { users: User[] }) {
  const [selected, setSelected] = useState<readonly string[]>([]);
  const [localUsers, setLocalUsers] = useState<User[]>(users);
  const theme = useTheme();
  const handleSelectAllClick = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (event.target.checked) {
      const newSelected = localUsers.map((n) => n.username);
      setSelected(newSelected);
      return;
    }
    setSelected([]);
  };
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

  const handleClick = (_event: React.MouseEvent<unknown>, username: string) => {
    const selectedIndex = selected.indexOf(username);
    let newSelected: readonly string[] = [];

    if (selectedIndex === -1) {
      newSelected = newSelected.concat(selected, username);
    } else if (selectedIndex === 0) {
      newSelected = newSelected.concat(selected.slice(1));
    } else if (selectedIndex === selected.length - 1) {
      newSelected = newSelected.concat(selected.slice(0, -1));
    } else if (selectedIndex > 0) {
      newSelected = newSelected.concat(
        selected.slice(0, selectedIndex),
        selected.slice(selectedIndex + 1),
      );
    }
    setSelected(newSelected);
  };
  const handleNewUser = () => {
    setLocalUsers((localUsers) => [
      ...localUsers,
      { username: `new user ${localUsers.length}`, roles: [] },
    ]);
  };
  const handleDelete = () => {
    setLocalUsers((localUsers) =>
      localUsers.filter((v) => !selected.find((s) => s === v.username)),
    );
    setSelected([]);
  };
  const setUsernameForIndex =
    (changedIndex: number) => (event: React.ChangeEvent<HTMLInputElement>) => {
      const newUsername = event.target.value;
      setLocalUsers((localUsers) =>
        localUsers.map(({ username, roles }, index) => {
          return index === changedIndex
            ? { username: newUsername, roles }
            : { username, roles };
        }),
      );
    };
  return (
    <Box sx={{ width: "100%" }}>
      <Paper sx={{ width: "100%", mb: 2 }}>
        <EnhancedTableToolbar
          numSelected={selected.length}
          onDelete={handleDelete}
        />
        <TableContainer>
          <Table aria-labelledby="tableTitle" size="medium">
            <caption>
              <Button onClick={handleNewUser}>Add User</Button>
              <Button style={{ float: "right" }} onClick={handleNewUser}>
                Save
              </Button>
            </caption>
            <EnhancedTableHead
              numSelected={selected.length}
              onSelectAllClick={handleSelectAllClick}
              rowCount={localUsers.length}
            />
            <TableBody>
              {localUsers.map(({ username, roles }, index) => {
                const isItemSelected = selected.includes(username);
                const labelId = `enhanced-table-checkbox-${index}`;
                return (
                  <TableRow
                    role="checkbox"
                    tabIndex={-1}
                    key={username}
                    selected={isItemSelected}
                    sx={{ cursor: "pointer" }}
                  >
                    <TableCell padding="checkbox">
                      <Checkbox
                        color="primary"
                        checked={isItemSelected}
                        onClick={(event) => handleClick(event, username)}
                        slotProps={{
                          input: {
                            "aria-labelledby": labelId,
                          },
                        }}
                      />
                    </TableCell>

                    <TableCell component="th" scope="row" padding="none">
                      <TextField
                        id="outlined-basic"
                        label="Username"
                        variant="outlined"
                        value={username}
                        onChange={setUsernameForIndex(index)}
                      />
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
                          <OutlinedInput
                            id="select-multiple-chip"
                            label="Roles"
                          />
                        }
                        renderValue={(selected) => (
                          <Box
                            sx={{ display: "flex", flexWrap: "wrap", gap: 0.5 }}
                          >
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
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>
    </Box>
  );
}
