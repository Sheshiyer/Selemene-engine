import { describe, expect, it } from "vitest";

import {
  buildDiscordCallbackOverride,
  shouldUseDynamicDiscordCallbackOverride
} from "./discord-oauth";

describe("discord oauth callback overrides", () => {
  it("does not override stable production hosts", () => {
    expect(
      shouldUseDynamicDiscordCallbackOverride("enantiodromia-engine-dashboard.vercel.app")
    ).toBe(false);
    expect(shouldUseDynamicDiscordCallbackOverride("144.tryambakam.space")).toBe(false);
    expect(
      buildDiscordCallbackOverride(
        "https://enantiodromia-engine-dashboard.vercel.app",
        "enantiodromia-engine-dashboard.vercel.app",
        "/admin/login/discord-callback"
      )
    ).toBeUndefined();
  });

  it("does not override preview hosts", () => {
    expect(shouldUseDynamicDiscordCallbackOverride("preview-123.vercel.app")).toBe(false);
    expect(
      buildDiscordCallbackOverride(
        "https://preview-123.vercel.app",
        "preview-123.vercel.app",
        "/admin/auth/discord/callback"
      )
    ).toBeUndefined();
  });

  it("allows localhost hosts to send an override", () => {
    expect(shouldUseDynamicDiscordCallbackOverride("localhost")).toBe(true);
    expect(
      buildDiscordCallbackOverride(
        "http://localhost:3001",
        "localhost",
        "/admin/auth/discord/callback"
      )
    ).toBe("http://localhost:3001/admin/auth/discord/callback");
  });

  it("normalizes trailing slashes in localhost callback paths", () => {
    expect(
      buildDiscordCallbackOverride(
        "http://localhost:3001",
        "localhost",
        "/admin/login/discord-callback///"
      )
    ).toBe("http://localhost:3001/admin/login/discord-callback");
  });
});