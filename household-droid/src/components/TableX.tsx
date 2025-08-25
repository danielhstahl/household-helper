import { useState } from "react";
import Box from "@mui/material/Box";
import Tooltip from "@mui/material/Tooltip";
import AddIcon from "@mui/icons-material/Add";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/DeleteOutlined";
import SaveIcon from "@mui/icons-material/Save";
import CancelIcon from "@mui/icons-material/Close";
import Button from "@mui/material/Button";
import OutlinedInput from "@mui/material/OutlinedInput";
import Chip from "@mui/material/Chip";
import MenuItem from "@mui/material/MenuItem";
import Select, { type SelectChangeEvent } from "@mui/material/Select";
import { type Theme, useTheme } from "@mui/material";
import {
  type GridRowsProp,
  type GridRowModesModel,
  GridRowModes,
  DataGrid,
  type GridColDef,
  GridActionsCellItem,
  type GridEventListener,
  type GridRowId,
  type GridRowModel,
  GridRowEditStopReasons,
  type GridSlotProps,
  Toolbar,
  ToolbarButton,
} from "@mui/x-data-grid";
import { useGridApiContext } from "@mui/x-data-grid";
import type { GridRenderEditCellParams } from "@mui/x-data-grid";
import { generate } from "random-words";

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

export const ActionEnum = {
  Update: "update",
  Create: "create",
  Delete: "delete",
} as const;
export type Action = (typeof ActionEnum)[keyof typeof ActionEnum];

declare module "@mui/x-data-grid" {
  interface ToolbarPropsOverrides {
    setRows: (newRows: (oldRows: GridRowsProp) => GridRowsProp) => void;
    setRowModesModel: (
      newModel: (oldModel: GridRowModesModel) => GridRowModesModel,
    ) => void;
    rows: GridRowsProp;
  }
}

const MultiSelect = (props: GridRenderEditCellParams) => {
  const { id, value, field } = props;
  const theme = useTheme();
  const apiRef = useGridApiContext();
  const handleValueChange = (event: SelectChangeEvent<string[]>) => {
    const newValue = event.target.value; // The new value entered by the user
    const newRoles =
      typeof newValue === "string" ? newValue.split(",") : newValue;

    apiRef.current.setEditCellValue({ id, field, value: newRoles });
  };
  return (
    <Select
      labelId="demo-multiple-chip-label"
      id="demo-multiple-chip"
      multiple
      value={value}
      onChange={handleValueChange}
      input={<OutlinedInput id="select-multiple-chip" label="Roles" />}
      renderValue={(selected) => (
        <Box sx={{ display: "flex", flexWrap: "wrap", gap: 0.5 }}>
          {selected.map((value: string) => (
            <Chip key={value} label={value} />
          ))}
        </Box>
      )}
      MenuProps={MenuProps}
    >
      {all_roles.map(({ value: role, label }) => (
        <MenuItem key={role} value={role} style={getStyles(role, value, theme)}>
          {label}
        </MenuItem>
      ))}
    </Select>
  );
};
const generatePassword = () => (generate(3) as string[]).join("");
function EditToolbar({
  rows,
  setRows,
  setRowModesModel,
}: GridSlotProps["toolbar"]) {
  const handleClick = () => {
    const id = rows.length + 1;
    setRows((oldRows) => [
      ...oldRows,
      {
        id,
        username: "",
        password: generatePassword(),
        roles: [],
        isNew: true,
      },
    ]);
    setRowModesModel((oldModel) => ({
      ...oldModel,
      [id]: { mode: GridRowModes.Edit, fieldToFocus: "username" },
    }));
  };

  return (
    <Toolbar>
      <Tooltip title="Add record">
        <ToolbarButton onClick={handleClick}>
          <AddIcon fontSize="small" />
        </ToolbarButton>
      </Tooltip>
    </Toolbar>
  );
}

const PasswordGenerate = ({ id, field }: GridRenderEditCellParams) => {
  const apiRef = useGridApiContext();
  const handleValueChange = () => {
    const newValue = generatePassword();
    apiRef.current.setEditCellValue({
      id,
      field,
      value: newValue,
    });
  };
  return (
    <Button variant="text" onClick={handleValueChange}>
      Regenerate Password
    </Button>
  );
};

export default function FullFeaturedCrudGrid({
  users,
  onChange,
}: {
  users: GridRowsProp;
  onChange: (
    type: Action,
    id: GridRowId,
    username: string,
    password: string,
    roles: string[],
  ) => void;
}) {
  const [rows, setRows] = useState(users);
  const [rowModesModel, setRowModesModel] = useState<GridRowModesModel>({});

  const handleRowEditStop: GridEventListener<"rowEditStop"> = (
    params,
    event,
  ) => {
    if (params.reason === GridRowEditStopReasons.rowFocusOut) {
      event.defaultMuiPrevented = true;
    }
  };

  const handleEditClick = (id: GridRowId) => () => {
    setRowModesModel({ ...rowModesModel, [id]: { mode: GridRowModes.Edit } });
  };

  const handleSaveClick = (id: GridRowId) => () => {
    setRowModesModel({ ...rowModesModel, [id]: { mode: GridRowModes.View } });
  };

  const handleDeleteClick = (id: GridRowId) => () => {
    const deletedRow = rows.find((row) => row.id === id);
    setRows(rows.filter((row) => row.id !== id));
    if (deletedRow) {
      onChange(
        ActionEnum.Delete,
        id,
        deletedRow.username,
        deletedRow.password,
        deletedRow.roles,
      );
    }
  };

  const handleCancelClick = (id: GridRowId) => () => {
    setRowModesModel({
      ...rowModesModel,
      [id]: { mode: GridRowModes.View, ignoreModifications: true },
    });

    const editedRow = rows.find((row) => row.id === id);
    if (editedRow!.isNew) {
      setRows(rows.filter((row) => row.id !== id));
    }
  };

  const processRowUpdate = (newRow: GridRowModel) => {
    const updatedRow = { ...newRow, isNew: false };
    setRows(rows.map((row) => (row.id === newRow.id ? updatedRow : row)));
    console.log(newRow);
    if (newRow.isNew) {
      onChange(
        ActionEnum.Create,
        newRow.id,
        newRow.username,
        newRow.password,
        newRow.roles,
      );
    } else {
      onChange(
        ActionEnum.Update,
        newRow.id,
        newRow.username,
        newRow.password,
        newRow.roles,
      );
    }

    return updatedRow;
  };

  const handleRowModesModelChange = (newRowModesModel: GridRowModesModel) => {
    setRowModesModel(newRowModesModel);
  };

  const columns: GridColDef[] = [
    {
      field: "username",
      headerName: "Username",
      flex: 1,
      editable: true,
    },
    {
      field: "roles",
      headerName: "Roles",
      type: "custom",
      flex: 2,
      editable: true,
      renderEditCell: (rowValue) => {
        return <MultiSelect {...rowValue} />;
      },
      renderCell: (value) => {
        return (
          <Box>
            {value.row.roles.map((value: string) => (
              <Chip key={value} label={value} />
            ))}
          </Box>
        );
      },
    },
    {
      field: "password",
      headerName: "Password",
      flex: 2,
      editable: true,
      type: "custom",
      renderEditCell: (rowValue) => {
        if (rowValue.row.isNew) {
          return rowValue.row.password;
        }
        return <PasswordGenerate {...rowValue} />;
      },
      renderCell: (value) => {
        return value.row.password || "*******";
      },
    },
    {
      field: "actions",
      type: "actions",
      headerName: "Actions",
      flex: 0.5,
      cellClassName: "actions",
      getActions: ({ id }) => {
        const isInEditMode = rowModesModel[id]?.mode === GridRowModes.Edit;

        if (isInEditMode) {
          return [
            <GridActionsCellItem
              icon={<SaveIcon />}
              label="Save"
              material={{
                sx: {
                  color: "primary.main",
                },
              }}
              onClick={handleSaveClick(id)}
            />,
            <GridActionsCellItem
              icon={<CancelIcon />}
              label="Cancel"
              className="textPrimary"
              onClick={handleCancelClick(id)}
              color="inherit"
            />,
          ];
        }

        return [
          <GridActionsCellItem
            icon={<EditIcon />}
            label="Edit"
            className="textPrimary"
            onClick={handleEditClick(id)}
            color="inherit"
          />,
          <GridActionsCellItem
            icon={<DeleteIcon />}
            label="Delete"
            onClick={handleDeleteClick(id)}
            color="inherit"
          />,
        ];
      },
    },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      <DataGrid
        rows={rows}
        columns={columns}
        editMode="row"
        rowModesModel={rowModesModel}
        onRowModesModelChange={handleRowModesModelChange}
        onRowEditStop={handleRowEditStop}
        processRowUpdate={processRowUpdate}
        slots={{ toolbar: EditToolbar }}
        slotProps={{
          toolbar: { rows, setRows, setRowModesModel },
        }}
        showToolbar
      />
    </div>
  );
}
