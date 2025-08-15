import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import App from "../App.tsx";
// All tests within this suite will be started in parallel
describe("App", () => {
  it("renders", async () => {
    const screen = render(<App />);
    await expect
      .element(screen.getByText(/Household Droid/i))
      .toBeInTheDocument();
  });
});
