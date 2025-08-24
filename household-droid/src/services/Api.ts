const buildUrl = (base: string, sessionId: string | undefined) => {
  return sessionId
    ? `${base}?` +
        new URLSearchParams({
          session_id: sessionId,
        })
    : base;
};
const getHeaders = (jwt: string) => ({
  "Content-Type": "application/json",
  Authorization: `Bearer ${jwt}`,
});

export const sendQuery = (text: string, jwt: string, sessionId: string) => {
  return fetch(buildUrl("/query", sessionId), {
    method: "POST",
    body: JSON.stringify({ text }),
    headers: getHeaders(jwt),
  });
  // Retrieve its body as ReadableStream
  /* .then((response) => {
        return response.body!.getReader();
      })*/
};

export const sendTutor = (text: string, jwt: string, sessionId: string) => {
  return fetch(buildUrl("/tutor", sessionId), {
    method: "POST",
    body: JSON.stringify({ text }),
    headers: getHeaders(jwt),
  });
  // Retrieve its body as ReadableStream
  /*.then((response) => {
        return response.body!.getReader();
      })*/
};

export const getSessions = (jwt: string) => {
  return fetch("/session", {
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const createSession = (jwt: string) => {
  return fetch("/session", {
    method: "POST",
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const getMessages = (sessionId: string, jwt: string) => {
  return fetch(`/messages/${sessionId}`, {
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};
export const getUser = (jwt: string) => {
  return fetch("/users/me", {
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};
export const getUsers = (jwt: string) => {
  return fetch("/users", {
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const createUser = (
  username: string,
  password: string,
  roles: string[],
  jwt: string,
) => {
  return fetch("/users", {
    method: "POST",
    body: JSON.stringify({ username, password, roles }),
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const updateUser = (
  id: number,
  username: string,
  password: string | undefined,
  roles: string[],
  jwt: string,
) => {
  const payload = password
    ? { id, username, password, roles }
    : { id, username, roles };
  return fetch("/users", {
    method: "PATCH",
    body: JSON.stringify(payload),
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const deleteUser = (
  id: number,
  username: string,
  password: string | undefined,
  jwt: string,
) => {
  return fetch("/users", {
    method: "DELETE",
    body: JSON.stringify({ id, username, password, roles: [] }),
    headers: getHeaders(jwt),
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const getToken = (formData: FormData) => {
  //https://github.com/microsoft/TypeScript/issues/30584#issuecomment-1865354582
  const data = new URLSearchParams(
    formData as unknown as Record<string, string>,
  );

  return fetch("/token", {
    method: "POST",
    body: data,
  }).then((response) => {
    return response.json().then((result) => {
      if (response.ok) {
        return result;
      }
      throw new Error(result.detail);
    });
  });
};

export const streamText = (
  onNewText: (_: string) => void,
  onDone: () => void,
) => {
  return async (r: ReadableStreamDefaultReader) => {
    let done = false;
    let value;
    const dec = new TextDecoder();
    while (!done) {
      ({ value, done } = await r.read());
      const strVal = dec.decode(value, { stream: true });
      onNewText(strVal);
    }
    onDone();
  };
};
