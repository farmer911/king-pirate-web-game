import { test, expect } from "@playwright/test";

// Fails fast when the game client cannot load and reach the game server within 10 s.
// "Reached" = the client finished its startup (XML/config init) and the server answered
// over the WebSocket bridge; for a new account the character-creation screen shows then.
// Tracing records screenshots of the busy Ruffle canvas and roughly doubles the load time, so it is off here.
test.use({ trace: "off" });

test("game client loads and talks to the server within 10 s", async ({ page }) => {
  test.setTimeout(20_000);
  const user = `load${Date.now() % 1_000_000}`;
  const form = (data: Record<string, string>) => ({
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    data: new URLSearchParams(data).toString(),
  });
  // Same flow as a player: the portal page (where they log in) loads first.
  await page.goto("/");
  await page.request.post("/api/auth/register", form({ Uname: user, Upass: `pw-${user}`, Scode: "1234" }));
  const login = await page.request.post("/api/auth/login", form({ Uname: user, Upass: `pw-${user}` }));
  expect((await login.json()).status).toBe("y");

  let received = 0;
  let start = Date.now();
  const reached = new Promise<void>((resolve) => {
    page.on("websocket", (ws) => {
      console.log(`ws open at ${((Date.now() - start) / 1000).toFixed(1)} s`);
      ws.on("framereceived", () => {
        if (++received >= 2) resolve();
      });
    });
  });
  start = Date.now();
  await page.goto("/?view=game&server=1");
  await Promise.race([reached, page.waitForTimeout(10_000)]);
  const seconds = (Date.now() - start) / 1000;
  expect(received, `server replies after ${seconds.toFixed(1)} s`).toBeGreaterThanOrEqual(2);
  expect(seconds).toBeLessThan(10);
});
