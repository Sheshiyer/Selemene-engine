---
name: selemene-onboarding
description: Introduce Selemene to new users, help them choose a reflection focus, and onboard them for their first free reading.
---

# Selemene Onboarding - Free Hosted Reflection

This skill is designed for first-time users to introduce them to Selemene's free hosted reflection service, guide them through choosing a reflection focus, and prepare them for their first reading.

## 1. Introduction and Focus Selection

Begin by welcoming the user and explaining that Selemene offers free symbolic reflection through a hosted service. Emphasize that it uses their current ChatGPT model for interpretation and synthesis.

- **Discover Reflection Engines:** Use `selemene_list_engines` to show the returned reflection engines and explain their returned descriptions. Number symbolism and mathematical cycle models are reflection frameworks, not measured traits or bodily energy. Catalog presence alone does not establish operation; never promise a fixed engine count.
- **Choose a Focus:** Ask the user to choose an engine or type of reflection they are interested in. If they have a specific question or area of life they want to explore, collect this information. This helps tailor the subsequent steps.
- **Language and Depth:** Use the conversation's language when clear; otherwise ask their preference. Offer a brief overview or detailed reading without presenting depth as a spiritual rank or access level. ChatGPT composes all report prose using the user's current model and applicable limits.

## 2. Input Collection and Tool Discovery

Based on the user's chosen reflection focus, guide them in providing the necessary inputs.

- **Engine-specific Inputs:** Use `selemene_engine_info` for the selected schema and `selemene_prepare_reading` to check supplied fields. Collect only actual requirements. Do not invent names, birth times, coordinates, timezones, dates, assessment answers or consent. A city label does not supply verified coordinates/timezone; preserve stated birth-time uncertainty.
- **Optional Context:** Offer to collect optional subject roles or context if relevant to their chosen reflection (e.g., relationship type for a compatibility reading). Clarify that these are for contextualizing the interpretation by ChatGPT and may not be directly sent to the tool.
- **Tool Catalog:** If the user expresses interest in combined workflows, use `selemene_list_workflows` to show them the available options and then `selemene_workflow_info` to explain what each workflow entails and its inputs.

## 3. Free Service Disclosure and Hand-off

Before any calculation or full reading request, ensure the user understands the nature of the free hosted service.

- **Shared Service Disclosure:** Clearly explain: "Free readings send submitted inputs to the hosted Selemene service. The service can store submitted inputs and reading results under a shared service/admin account and may update its profile, history, usage, or XP. This does not create a personal linked account or provide private account history." Reinforce that no personal API key or login is required.
- **Execution scope:** Preparing and discussing a reading never authorizes a calculation. When the user explicitly requests their selected reading after understanding the disclosure, continue without requesting the same authorization again. Keep free quota limits and any unavailable service state visible.
- **Hand-off to `selemene-reading`:** Once the user selects a reading, use `selemene-reading` for its preflight, explicitly requested execution, and host-model synthesis. Onboarding owns introduction and selection; it does not independently repeat execution or invoke a hosted narration/report endpoint.
