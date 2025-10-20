import { describe, it, expect, vi } from "vitest";
import { render } from "vitest-browser-react";
import Table, { ActionEnum } from "../TableX.tsx";
import { RoleTypeEnum } from "../../services/models.tsx";

describe("Table", () => {
  it("renders", async () => {
    const users = [{ id: 3, roles: [RoleTypeEnum.admin], username: "hello" }];
    const onChange = vi.fn();
    const screen = render(<Table users={users} onChange={onChange} />);
    await expect.element(screen.getByText(/Username/i)).toBeInTheDocument();
    await expect.element(screen.getByText(/hello/i)).toBeInTheDocument();
  });
  it("adds new user on add user", async () => {
    const users = [{ id: 3, roles: [RoleTypeEnum.admin], username: "hello" }];
    const onChange = vi.fn();
    const screen = render(<Table users={users} onChange={onChange} />);
    await expect
      .element(screen.getByRole("grid"))
      .toHaveAttribute("aria-rowcount", "2"); //header counts as row
    const addButton = screen.getByRole("button", { name: "AddButton" });
    await addButton.click();
    await expect
      .element(screen.getByRole("grid"))
      .toHaveAttribute("aria-rowcount", "3");
  });
  it("edits user on edit user", async () => {
    const users = [{ id: 3, roles: [RoleTypeEnum.admin], username: "hello" }];
    const onChange = vi.fn();
    const screen = render(<Table users={users} onChange={onChange} />);
    const editButton = screen.getByRole("menuitem", { name: "EditButton" });
    await editButton.click();
    await expect
      .element(screen.getByRole("button", { name: "Regenerate Password" }))
      .toBeInTheDocument();
    const regeneratePassword = screen.getByRole("button", {
      name: "Regenerate Password",
    });
    await regeneratePassword.click();
    await expect
      .element(screen.getByRole("menuitem", { name: "SaveButton" }))
      .toBeInTheDocument();
    const saveButton = screen.getByRole("menuitem", { name: "SaveButton" });
    await saveButton.click();
    expect(onChange.mock.calls.length).toEqual(1);
    const [calledWith] = onChange.mock.calls;
    expect(calledWith.length).toEqual(5);
    expect(calledWith[0]).toEqual(ActionEnum.Update);
    expect(calledWith[1]).toEqual(3);
    expect(calledWith[2]).toEqual("hello");
    expect(calledWith[4]).toEqual(["admin"]);
  });
});
