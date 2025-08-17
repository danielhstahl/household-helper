export const sendQuery = (text: string) => {
  return (
    fetch("/query", {
      method: "POST",
      body: JSON.stringify({ text }),
      headers: { "Content-Type": "application/json" },
    })
      // Retrieve its body as ReadableStream
      .then((response) => {
        return response.body!.getReader();
      })
  );
};

export const sendTutor = (text: string) => {
  return (
    fetch("/tutor", {
      method: "POST",
      body: JSON.stringify({ text }),
      headers: { "Content-Type": "application/json" },
    })
      // Retrieve its body as ReadableStream
      .then((response) => {
        return response.body!.getReader();
      })
  );
};

export const getToken = (formData: FormData) => {
  //https://github.com/microsoft/TypeScript/issues/30584#issuecomment-1865354582
  const data = new URLSearchParams(
    formData as unknown as Record<string, string>,
  );

  return fetch("/token", {
    method: "POST",
    body: data,
    //headers: { "Content-Type": "application/json" },
  }).then((response) => {
    return response.json();
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
