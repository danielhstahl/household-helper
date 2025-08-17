import FormControl from "@mui/material/FormControl";
import OutlinedInput from "@mui/material/OutlinedInput";
import { Form } from "react-router";
const Auth = () => {
  return (
    <Form
      noValidate
      autoComplete="off"
      method="post"
      action={`/`} //go to main page, where the "Action" will be triggered to get a token
    >
      <FormControl sx={{ width: "25ch" }}>
        <OutlinedInput placeholder="Username" name="username" />
        <OutlinedInput placeholder="Password" type="password" name="password" />
      </FormControl>
    </Form>
  );
};

export default Auth;
