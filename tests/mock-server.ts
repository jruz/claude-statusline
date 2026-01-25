const server = Bun.serve({
  port: 3000,
  fetch(req) {
    const url = new URL(req.url);
    const scenario = url.searchParams.get("scenario");

    if (url.pathname === "/v1/rate_limits") {
      if (scenario === "error") {
        return new Response("Internal Server Error", { status: 500 });
      }
      if (scenario === "timeout") {
        return new Promise(() => {}); // never resolves
      }
      if (scenario === "malformed") {
        return new Response("not json", { status: 200 });
      }
      if (scenario === "empty") {
        return Response.json({ rate_limits: [] });
      }
      return Response.json({
        rate_limits: [
          { type: "requests", utilization: 0.6, resets_at: "2026-01-25T15:30:00Z" },
          { type: "tokens", utilization: 0.4, resets_at: "2026-01-30T00:00:00Z" },
        ],
      });
    }
    return new Response("Not Found", { status: 404 });
  },
});
