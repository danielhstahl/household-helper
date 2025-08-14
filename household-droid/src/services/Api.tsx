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
