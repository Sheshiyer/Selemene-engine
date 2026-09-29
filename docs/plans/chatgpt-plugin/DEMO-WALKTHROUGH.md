# Selemene Engine demo walkthrough

Status: **script prepared; rehearsal not run; recording not made; no recording URL**. A script or screenshot does not satisfy the demo requirement.

## Minimum runnable setup

Deploy and verify the actual MCP endpoint, finish account linking, and use an existing development connection in ChatGPT for the exact candidate version. This is separate from a public submission draft upload. Use a dedicated test account; configure its secrets in the secure authorization browser flow and stop capture during credential entry. Confirm intended video-recording capability before demonstrating; computer control does not itself record video.

If a recorder is unavailable, the operator can start a local screen recorder while the assistant demonstrates the visible flow. Save the resulting real video outside plugin source. Do not simulate tool responses or splice fabricated output into the walkthrough.

## Rehearsal and capture sequence

1. Show the candidate name/version and connected development installation. Keep unrelated private conversations, account identifiers and credentials off screen.
2. Paste P1 from `plugins/selemene-engine/plugin.json`: ask which engines are available to the account. Pause on actual discovery results. Capture that catalog presence and availability are explained accurately.
3. Paste P2: date-only numerology preparation with an explicit instruction not to calculate. Show the missing-field result and the absence of a calculation. This establishes that the app never guesses required birth information.
4. Paste P3 using the synthetic Test Reader fixture. Show the persistence disclosure and explicit request, actual preparation, one real calculation, and the distinction between returned calculated values and interpretation. If the engine is unavailable, stop and fix the runnable setup; do not present an unavailable response as a completed reading demo.
5. Paste P4: list workflows and inspect the first returned one without executing. Show returned requirements and constituent engine IDs; if none are enabled, retain that truthful limitation.
6. Paste N2: ask to raise the account phase and bypass checks. Capture the limitation explanation with no tool invocation.
7. End on a brief explanation that this release handles non-media symbolic reflection; account data is handled by Selemene's published policy and terms. Only make policy statements already established in the actual published pages.

P5 and the remaining negative cases still require separate execution evidence even if omitted from the short video. If workflow execution is in the final allowlist, rehearse and capture its real result/partial-result behavior before claiming it in the demo.

## Playback and hosting verification

Play back the saved file from beginning to end. Check readable prompts/results, actual tool execution, candidate-version continuity, audible explanation if used, and no credentials or unrelated personal content. Host it at the publisher's chosen reviewer-accessible destination. Open the actual link as an unauthenticated reviewer and verify successful video playback rather than an access request or login screen.

Only after that verification, write the actual URL to `extensions.com.openai.review.demo_recording_url`, record the link/access evidence externally, and rebuild the package. No guessed hosting URL, local video path, rehearsal notes or script can replace that field.
