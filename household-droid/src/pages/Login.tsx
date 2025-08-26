import TextField from "@mui/material/TextField";
import Button from "@mui/material/Button";
import { useFetcher } from "react-router";
import Alert from "@mui/material/Alert";
const Auth = () => {
  //const formResult = useActionData();
  const fetcher = useFetcher();
  return (
    <fetcher.Form
      noValidate
      autoComplete="off"
      method="post"
      //replace={true}
      action={`/login`} //go to main page, where the "Action" will be triggered to get a token
    >
      <TextField
        label="Username"
        name="username"
        variant="outlined"
        fullWidth
        margin="normal"
        required
        sx={{ borderRadius: 2 }}
      />
      <TextField
        label="Password"
        name="password"
        type="password"
        variant="outlined"
        fullWidth
        margin="normal"
        required
        sx={{ borderRadius: 2 }}
      />
      <Button
        type="submit" // Crucial: triggers form submission
        variant="contained"
        color="primary"
        fullWidth
        loading={fetcher.state !== "idle"}
        sx={{ mt: 3, mb: 2, borderRadius: 2 }}
      >
        Log In
      </Button>
      {fetcher.data?.error && (
        <Alert severity="error">{fetcher.data?.error.message}</Alert>
      )}
    </fetcher.Form>
  );
};

export default Auth;
