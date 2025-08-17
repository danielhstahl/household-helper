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
export const sendQuery = (
  text: string,
  jwt: string,
  sessionId: string | undefined,
) => {
  return (
    fetch(buildUrl("/query", sessionId), {
      method: "POST",
      body: JSON.stringify({ text }),
      headers: getHeaders(jwt),
    })
      // Retrieve its body as ReadableStream
      .then((response) => {
        return response.body!.getReader();
      })
  );
};

export const sendTutor = (
  text: string,
  jwt: string,
  sessionId: string | undefined,
) => {
  return (
    fetch(buildUrl("/tutor", sessionId), {
      method: "POST",
      body: JSON.stringify({ text }),
      headers: getHeaders(jwt),
    })
      // Retrieve its body as ReadableStream
      .then((response) => {
        return response.body!.getReader();
      })
  );
};

export const getSession = (jwt: string) => {
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
