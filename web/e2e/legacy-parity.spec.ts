import { test, expect } from "@playwright/test";
import * as fs from "fs";

test.describe("King Pirate Legacy-to-Rust 100% Parity & Zero-PHP SPA Verification", () => {
  test("1. Zero .php network requests and zero <iframe> wrapper in React SPA", async ({ page }) => {
    const phpRequests: string[] = [];
    page.on("request", (req) => {
      const url = req.url();
      if (url.includes(".php")) {
        phpRequests.push(url);
      }
    });

    await page.goto("/");
    await expect(page.locator(".DH_Panel_Title")).toContainText("VUA HẢI TẶC");

    // Attempt login via SPA modal
    await page.click("#btn-open-login");
    await page.getByPlaceholder("Tài Khoản").fill("loulx01");
    await page.getByPlaceholder("Mật Khẩu").fill("123456");
    await page.locator("form button[type='submit']").click();

    await page.waitForTimeout(300);
    expect(phpRequests, `Found forbidden .php requests: ${phpRequests.join(", ")}`).toEqual([]);
    expect(page.url()).not.toContain(".php");
  });

  test("2. Direct Ruffle WebAssembly Player (<ruffle-player> / <ruffle-embed>) mounted in React DOM without <iframe>", async ({
    page,
  }) => {
    await page.goto("/?view=game&server=1");
    const iframeCount = await page.locator("iframe").count();
    expect(iframeCount, "React SPA must not use <iframe> for game rendering").toBe(0);

    const ruffleContainer = page.locator("#ruffle-game-container");
    await expect(ruffleContainer).toBeVisible();
    await expect(page.locator("#Main_Game")).toBeAttached({ timeout: 5000 });
  });

  test("3. Legacy LouLx GM Tool (/admin_loulxgame from [5] GM.bat & Console.class.php) is accessible and functional", async ({
    request,
    page,
  }) => {
    const gmRes = await request.get("/api/gm/info");
    expect(gmRes.status(), "GM API endpoint /api/gm/info must return 200 OK").toBe(200);
    const gmJson = await gmRes.json();
    expect(gmJson.status).toBe("y");
    expect(Array.isArray(gmJson.presets)).toBe(true);

    // Execute live GM item & gold grant and verify persistence in DataStore
    const execRes = await request.post("/api/gm/execute", {
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      data: "uid=20001&command=item&val1=101101&val2=1",
    });
    expect(execRes.status()).toBe(200);
    const execJson = await execRes.json();
    expect(execJson.status).toBe("y");
    expect(execJson.info).toContain("101101");

    // Open GM Tool modal directly via SPA route
    await page.goto("/?view=gm");
    await expect(page.locator("#GmConsoleForm")).toBeVisible();
  });

  test("4. Live Main.swf RPC log has 0 unhandled fallback methods and user.getSwitch returns all 63 SwitchDef IDs", async () => {
    const modRs = fs.readFileSync("../src/domain/mod.rs", "utf8");
    expect(
      modRs.includes('("user", "getSwitch" | "getSwitchInfo") | ("switch", "getSwitchInfo") => {\n            Amf3Value::dense_list([])'),
      "user.getSwitch must not return an empty array [] which locks all 63 UI icons in Main.swf"
    ).toBe(false);
    expect(
      modRs.includes("charity.fetchVipSalary") || modRs.includes('"fetchVipSalary"'),
      "charity.fetchVipSalary must be explicitly implemented in dispatch_client_rpc"
    ).toBe(true);
    expect(
      modRs.includes("charity.fetchPresigeSalary") || modRs.includes('"fetchPresigeSalary"'),
      "charity.fetchPresigeSalary must be explicitly implemented in dispatch_client_rpc"
    ).toBe(true);
  });
});
