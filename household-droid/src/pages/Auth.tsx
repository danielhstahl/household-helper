import TextField from "@mui/material/TextField";
import Button from "@mui/material/Button";
import { Form, useActionData } from "react-router";
import Alert from "@mui/material/Alert";
const Auth = () => {
  const formResult = useActionData();
  return (
    <Form
      noValidate
      autoComplete="off"
      method="post"
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
        sx={{ mt: 3, mb: 2, borderRadius: 2 }}
      >
        Log In
      </Button>
      {formResult?.error && (
        <Alert severity="error">{formResult.error.message}</Alert>
      )}
    </Form>
  );
};

export default Auth;
