import FormControl from "@mui/material/FormControl";
import OutlinedInput from "@mui/material/OutlinedInput";
const Auth = () => {
  return (
    <form noValidate autoComplete="off" onSubmit={(e) => console.log(e)}>
      <FormControl sx={{ width: "25ch" }}>
        <OutlinedInput placeholder="Username" />
        <OutlinedInput placeholder="Password" type="password" />
      </FormControl>
    </form>
  );
};

export default Auth;
